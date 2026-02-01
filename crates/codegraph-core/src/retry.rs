//! Retry policies with exponential backoff and circuit breaker
//!
//! Provides configurable retry behavior for external API calls.
//!
//! # Configuration
//!
//! Set the following environment variables:
//! - `RETRY_MAX_OPENAI`: Max retries for OpenAI (default: 3)
//! - `RETRY_MAX_DB`: Max retries for databases (default: 2)
//! - `RETRY_BASE_DELAY_MS`: Base delay in milliseconds (default: 100)
//! - `CIRCUIT_BREAKER_THRESHOLD`: Consecutive failures to open circuit (default: 5)
//! - `CIRCUIT_BREAKER_TIMEOUT_SECS`: Time before attempting reset (default: 30)

use std::env;
use std::future::Future;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

use tracing::{error, info, warn};

/// Type of service for retry configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceType {
    /// OpenAI API calls (max 3 retries)
    OpenAI,
    /// Database operations (max 2 retries)
    Database,
    /// Generic external API (max 3 retries)
    ExternalApi,
}

impl ServiceType {
    /// Get the maximum number of retries for this service type
    pub fn max_retries(&self) -> u32 {
        match self {
            ServiceType::OpenAI => env::var("RETRY_MAX_OPENAI")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            ServiceType::Database => env::var("RETRY_MAX_DB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
            ServiceType::ExternalApi => 3,
        }
    }

    /// Get the base delay for this service type
    pub fn base_delay(&self) -> Duration {
        let base_ms = env::var("RETRY_BASE_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100u64);

        match self {
            ServiceType::OpenAI => Duration::from_millis(base_ms * 2), // Start slower for rate limits
            ServiceType::Database => Duration::from_millis(base_ms),
            ServiceType::ExternalApi => Duration::from_millis(base_ms),
        }
    }

    /// Get the timeout for this service type
    pub fn timeout(&self) -> Duration {
        match self {
            ServiceType::OpenAI => Duration::from_secs(60),   // LLM calls can be slow
            ServiceType::Database => Duration::from_secs(10), // DB should be fast
            ServiceType::ExternalApi => Duration::from_secs(30),
        }
    }
}

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay between retries (will be multiplied by 2^attempt)
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Operation timeout
    pub timeout: Duration,
    /// Add jitter to prevent thundering herd
    pub jitter: bool,
}

impl RetryPolicy {
    /// Create a policy for the given service type
    pub fn for_service(service: ServiceType) -> Self {
        Self {
            max_retries: service.max_retries(),
            base_delay: service.base_delay(),
            max_delay: Duration::from_secs(30),
            timeout: service.timeout(),
            jitter: true,
        }
    }

    /// Create a custom retry policy
    pub fn custom(max_retries: u32, base_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay: Duration::from_secs(30),
            timeout: Duration::from_secs(30),
            jitter: true,
        }
    }

    /// Set the maximum delay
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enable or disable jitter
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Calculate delay for the given attempt (0-indexed)
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay = self.base_delay * 2u32.saturating_pow(attempt);
        let delay = delay.min(self.max_delay);

        if self.jitter {
            // Add up to 25% jitter
            let jitter_factor = 1.0 + (rand_simple() * 0.25);
            Duration::from_secs_f64(delay.as_secs_f64() * jitter_factor)
        } else {
            delay
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::for_service(ServiceType::ExternalApi)
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests are allowed
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, allowing one test request
    HalfOpen,
}

/// Circuit breaker for preventing cascading failures
pub struct CircuitBreaker {
    /// Current state
    state: RwLock<CircuitState>,
    /// Number of consecutive failures
    failure_count: AtomicU32,
    /// Threshold for opening the circuit
    failure_threshold: u32,
    /// Time when the circuit was opened
    opened_at: RwLock<Option<Instant>>,
    /// Time to wait before attempting to close
    reset_timeout: Duration,
    /// Service name for logging
    service_name: String,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default settings
    pub fn new(service_name: impl Into<String>) -> Self {
        let failure_threshold = env::var("CIRCUIT_BREAKER_THRESHOLD")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        let reset_timeout_secs = env::var("CIRCUIT_BREAKER_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30u64);

        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            failure_threshold,
            opened_at: RwLock::new(None),
            reset_timeout: Duration::from_secs(reset_timeout_secs),
            service_name: service_name.into(),
        }
    }

    /// Create with custom settings
    pub fn with_settings(
        service_name: impl Into<String>,
        failure_threshold: u32,
        reset_timeout: Duration,
    ) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            failure_threshold,
            opened_at: RwLock::new(None),
            reset_timeout,
            service_name: service_name.into(),
        }
    }

    /// Get the current state
    pub fn state(&self) -> CircuitState {
        let state = *self.state.read().unwrap();

        // Check if we should transition from Open to HalfOpen
        if state == CircuitState::Open {
            if let Some(opened_at) = *self.opened_at.read().unwrap() {
                if opened_at.elapsed() >= self.reset_timeout {
                    let mut state_guard = self.state.write().unwrap();
                    *state_guard = CircuitState::HalfOpen;
                    info!(
                        service = %self.service_name,
                        "Circuit breaker transitioning to half-open"
                    );
                    return CircuitState::HalfOpen;
                }
            }
        }

        state
    }

    /// Check if requests are allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self.state(), CircuitState::Closed | CircuitState::HalfOpen)
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);

        let mut state = self.state.write().unwrap();
        if *state == CircuitState::HalfOpen {
            *state = CircuitState::Closed;
            *self.opened_at.write().unwrap() = None;
            info!(
                service = %self.service_name,
                "Circuit breaker closed after successful request"
            );
        }
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;

        let mut state = self.state.write().unwrap();
        if *state == CircuitState::HalfOpen {
            // Failed during half-open, reopen immediately
            *state = CircuitState::Open;
            *self.opened_at.write().unwrap() = Some(Instant::now());
            warn!(
                service = %self.service_name,
                "Circuit breaker reopened after half-open failure"
            );
        } else if failures >= self.failure_threshold && *state == CircuitState::Closed {
            // Threshold reached, open the circuit
            *state = CircuitState::Open;
            *self.opened_at.write().unwrap() = Some(Instant::now());
            error!(
                service = %self.service_name,
                failures = failures,
                threshold = self.failure_threshold,
                "Circuit breaker opened due to consecutive failures"
            );
        }
    }

    /// Reset the circuit breaker
    pub fn reset(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        *self.state.write().unwrap() = CircuitState::Closed;
        *self.opened_at.write().unwrap() = None;
        info!(service = %self.service_name, "Circuit breaker reset");
    }
}

/// Error indicating circuit is open
#[derive(Debug, Clone)]
pub struct CircuitOpenError {
    pub service: String,
}

impl std::fmt::Display for CircuitOpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Circuit breaker is open for service: {}", self.service)
    }
}

impl std::error::Error for CircuitOpenError {}

/// Retry result with metrics
#[derive(Debug)]
pub struct RetryResult<T> {
    /// The result value
    pub value: T,
    /// Number of attempts made
    pub attempts: u32,
    /// Total time spent
    pub total_duration: Duration,
}

/// Execute an async operation with retry policy
pub async fn with_retry<F, Fut, T, E>(
    policy: &RetryPolicy,
    operation_name: &str,
    mut operation: F,
) -> Result<RetryResult<T>, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let start = Instant::now();
    let mut attempts = 0;

    loop {
        attempts += 1;

        match operation().await {
            Ok(value) => {
                if attempts > 1 {
                    info!(
                        operation = operation_name,
                        attempts = attempts,
                        duration_ms = %start.elapsed().as_millis(),
                        "Operation succeeded after retries"
                    );
                }
                return Ok(RetryResult {
                    value,
                    attempts,
                    total_duration: start.elapsed(),
                });
            }
            Err(e) if attempts <= policy.max_retries => {
                let delay = policy.delay_for_attempt(attempts - 1);
                warn!(
                    operation = operation_name,
                    attempt = attempts,
                    max_retries = policy.max_retries,
                    delay_ms = %delay.as_millis(),
                    error = %e,
                    "Operation failed, retrying"
                );
                tokio::time::sleep(delay).await;
            }
            Err(e) => {
                error!(
                    operation = operation_name,
                    attempts = attempts,
                    duration_ms = %start.elapsed().as_millis(),
                    error = %e,
                    "Operation failed after all retries"
                );
                return Err(e);
            }
        }
    }
}

/// Execute an async operation with retry policy and circuit breaker
pub async fn with_retry_and_circuit_breaker<F, Fut, T, E>(
    policy: &RetryPolicy,
    circuit: &CircuitBreaker,
    operation_name: &str,
    operation: F,
) -> Result<RetryResult<T>, RetryError<E>>
where
    F: FnMut() -> Fut + Clone,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    // Check circuit breaker first
    if !circuit.is_allowed() {
        return Err(RetryError::CircuitOpen(CircuitOpenError {
            service: circuit.service_name.clone(),
        }));
    }

    match with_retry(policy, operation_name, operation).await {
        Ok(result) => {
            circuit.record_success();
            Ok(result)
        }
        Err(e) => {
            circuit.record_failure();
            Err(RetryError::OperationFailed(e))
        }
    }
}

/// Error type for retry with circuit breaker
#[derive(Debug)]
pub enum RetryError<E> {
    /// Circuit breaker is open
    CircuitOpen(CircuitOpenError),
    /// Operation failed after all retries
    OperationFailed(E),
}

impl<E: std::fmt::Display> std::fmt::Display for RetryError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryError::CircuitOpen(e) => write!(f, "{}", e),
            RetryError::OperationFailed(e) => write!(f, "Operation failed: {}", e),
        }
    }
}

impl<E: std::fmt::Display + std::fmt::Debug> std::error::Error for RetryError<E> {}

/// Simple pseudo-random number generator (0.0 to 1.0)
/// Used for jitter without requiring rand crate
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as f64;
    (nanos % 1000.0) / 1000.0
}

/// Retry metrics for monitoring
#[derive(Debug, Default)]
pub struct RetryMetrics {
    /// Total operations attempted
    pub total_operations: AtomicU32,
    /// Operations that succeeded on first try
    pub first_try_success: AtomicU32,
    /// Operations that succeeded after retry
    pub retry_success: AtomicU32,
    /// Operations that failed after all retries
    pub total_failures: AtomicU32,
    /// Total retries across all operations
    pub total_retries: AtomicU32,
}

impl RetryMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful operation
    pub fn record_success(&self, attempts: u32) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        if attempts == 1 {
            self.first_try_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.retry_success.fetch_add(1, Ordering::Relaxed);
            self.total_retries
                .fetch_add(attempts - 1, Ordering::Relaxed);
        }
    }

    /// Record a failed operation
    pub fn record_failure(&self, attempts: u32) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.total_failures.fetch_add(1, Ordering::Relaxed);
        self.total_retries.fetch_add(attempts, Ordering::Relaxed);
    }

    /// Get success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        let total = self.total_operations.load(Ordering::Relaxed);
        if total == 0 {
            return 1.0;
        }
        let failures = self.total_failures.load(Ordering::Relaxed);
        (total - failures) as f64 / total as f64
    }

    /// Get average retries per operation
    pub fn avg_retries(&self) -> f64 {
        let total = self.total_operations.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        self.total_retries.load(Ordering::Relaxed) as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_delay() {
        let policy = RetryPolicy::custom(3, Duration::from_millis(100)).with_jitter(false);

        assert_eq!(policy.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(400));
    }

    #[test]
    fn test_circuit_breaker_state_transitions() {
        let cb = CircuitBreaker::with_settings("test", 2, Duration::from_millis(10));

        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.is_allowed());

        // First failure
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        // Second failure opens circuit
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.is_allowed());

        // Reset
        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_service_type_config() {
        assert!(ServiceType::OpenAI.max_retries() >= 1);
        assert!(ServiceType::Database.max_retries() >= 1);
        assert!(ServiceType::OpenAI.base_delay() > Duration::ZERO);
    }

    #[test]
    fn test_retry_metrics() {
        let metrics = RetryMetrics::new();

        metrics.record_success(1);
        metrics.record_success(2);
        metrics.record_failure(3);

        assert_eq!(metrics.total_operations.load(Ordering::Relaxed), 3);
        assert_eq!(metrics.first_try_success.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.retry_success.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_failures.load(Ordering::Relaxed), 1);
    }
}
