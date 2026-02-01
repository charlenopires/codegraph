//! Graceful degradation for service outages
//!
//! Provides mechanisms for detecting service failures and operating in degraded mode.
//!
//! # Features
//!
//! - Service health tracking with automatic degradation
//! - Cached response fallback
//! - Clear degradation indicators in responses
//! - Structured logging for monitoring

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Service identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Service {
    Neo4j,
    Qdrant,
    Redis,
    OpenAI,
    Ona,
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Service::Neo4j => write!(f, "neo4j"),
            Service::Qdrant => write!(f, "qdrant"),
            Service::Redis => write!(f, "redis"),
            Service::OpenAI => write!(f, "openai"),
            Service::Ona => write!(f, "ona"),
        }
    }
}

/// Service health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthState {
    /// Service is operational
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is offline
    Offline,
    /// Service status is unknown
    Unknown,
}

/// Information about a service's health
#[derive(Debug, Clone, Serialize)]
pub struct ServiceHealth {
    /// Current health state
    pub state: HealthState,
    /// Last successful check time
    pub last_success: Option<String>,
    /// Last error message
    pub last_error: Option<String>,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Whether the service is required for core functionality
    pub is_required: bool,
}

/// Operating mode of the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatingMode {
    /// All services operational
    Normal,
    /// Some non-critical services offline
    Degraded,
    /// Critical services offline, using cached responses
    Cached,
    /// System is offline
    Offline,
}

impl OperatingMode {
    /// Check if the mode allows serving requests
    pub fn can_serve(&self) -> bool {
        !matches!(self, OperatingMode::Offline)
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            OperatingMode::Normal => "All services operational",
            OperatingMode::Degraded => "Running with limited functionality",
            OperatingMode::Cached => "Serving cached responses only",
            OperatingMode::Offline => "System is offline",
        }
    }
}

/// Degradation manager tracks service health and determines operating mode
pub struct DegradationManager {
    /// Health state of each service
    services: RwLock<HashMap<Service, ServiceHealthState>>,
    /// Current operating mode
    mode: RwLock<OperatingMode>,
    /// Whether degraded mode is enabled
    degradation_enabled: AtomicBool,
    /// Total requests served in degraded mode
    degraded_requests: AtomicU64,
}

struct ServiceHealthState {
    state: HealthState,
    last_success: Option<Instant>,
    last_error: Option<String>,
    failure_count: u32,
    is_required: bool,
}

impl DegradationManager {
    /// Create a new degradation manager
    pub fn new() -> Self {
        let mut services = HashMap::new();

        // Initialize service health states
        for service in [
            Service::Neo4j,
            Service::Qdrant,
            Service::Redis,
            Service::OpenAI,
            Service::Ona,
        ] {
            let is_required = matches!(service, Service::Neo4j);
            services.insert(
                service,
                ServiceHealthState {
                    state: HealthState::Unknown,
                    last_success: None,
                    last_error: None,
                    failure_count: 0,
                    is_required,
                },
            );
        }

        Self {
            services: RwLock::new(services),
            mode: RwLock::new(OperatingMode::Normal),
            degradation_enabled: AtomicBool::new(true),
            degraded_requests: AtomicU64::new(0),
        }
    }

    /// Enable or disable degradation mode
    pub fn set_degradation_enabled(&self, enabled: bool) {
        self.degradation_enabled.store(enabled, Ordering::SeqCst);
        info!(
            enabled = enabled,
            "Graceful degradation {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Check if degradation is enabled
    pub fn is_degradation_enabled(&self) -> bool {
        self.degradation_enabled.load(Ordering::SeqCst)
    }

    /// Record a successful service operation
    pub fn record_success(&self, service: Service) {
        let mut services = self.services.write().unwrap();
        if let Some(state) = services.get_mut(&service) {
            let was_unhealthy = state.state != HealthState::Healthy;
            state.state = HealthState::Healthy;
            state.last_success = Some(Instant::now());
            state.failure_count = 0;
            state.last_error = None;

            if was_unhealthy {
                info!(
                    service = %service,
                    "Service recovered"
                );
            }
        }
        drop(services);
        self.update_mode();
    }

    /// Record a failed service operation
    pub fn record_failure(&self, service: Service, error: impl Into<String>) {
        let error = error.into();
        let mut services = self.services.write().unwrap();
        if let Some(state) = services.get_mut(&service) {
            state.failure_count += 1;
            state.last_error = Some(error.clone());

            // Transition to degraded after 3 failures, offline after 5
            state.state = if state.failure_count >= 5 {
                HealthState::Offline
            } else if state.failure_count >= 3 {
                HealthState::Degraded
            } else {
                HealthState::Healthy
            };

            if state.state != HealthState::Healthy {
                warn!(
                    service = %service,
                    failure_count = state.failure_count,
                    error = %error,
                    state = ?state.state,
                    "Service degradation"
                );
            }
        }
        drop(services);
        self.update_mode();
    }

    /// Get the current operating mode
    pub fn mode(&self) -> OperatingMode {
        *self.mode.read().unwrap()
    }

    /// Get health status for a specific service
    pub fn service_health(&self, service: Service) -> ServiceHealth {
        let services = self.services.read().unwrap();
        services.get(&service).map_or(
            ServiceHealth {
                state: HealthState::Unknown,
                last_success: None,
                last_error: None,
                failure_count: 0,
                is_required: false,
            },
            |s| ServiceHealth {
                state: s.state,
                last_success: s.last_success.map(|t| {
                    let elapsed = t.elapsed();
                    format!("{}s ago", elapsed.as_secs())
                }),
                last_error: s.last_error.clone(),
                failure_count: s.failure_count,
                is_required: s.is_required,
            },
        )
    }

    /// Get all service health statuses
    pub fn all_services_health(&self) -> HashMap<Service, ServiceHealth> {
        let services = self.services.read().unwrap();
        services
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    ServiceHealth {
                        state: v.state,
                        last_success: v.last_success.map(|t| {
                            let elapsed = t.elapsed();
                            format!("{}s ago", elapsed.as_secs())
                        }),
                        last_error: v.last_error.clone(),
                        failure_count: v.failure_count,
                        is_required: v.is_required,
                    },
                )
            })
            .collect()
    }

    /// Check if a service is available
    pub fn is_service_available(&self, service: Service) -> bool {
        let services = self.services.read().unwrap();
        services
            .get(&service)
            .map_or(false, |s| s.state != HealthState::Offline)
    }

    /// Check if the system can serve requests
    pub fn can_serve(&self) -> bool {
        self.mode().can_serve()
    }

    /// Record a request served in degraded mode
    pub fn record_degraded_request(&self) {
        self.degraded_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Get count of degraded requests
    pub fn degraded_request_count(&self) -> u64 {
        self.degraded_requests.load(Ordering::Relaxed)
    }

    /// Get degradation status for API responses
    pub fn degradation_status(&self) -> DegradationStatus {
        DegradationStatus {
            mode: self.mode(),
            is_degraded: self.mode() != OperatingMode::Normal,
            message: self.mode().description().to_string(),
            degraded_services: self.get_degraded_services(),
            degraded_request_count: self.degraded_request_count(),
        }
    }

    /// Update the operating mode based on service health
    fn update_mode(&self) {
        let services = self.services.read().unwrap();

        let mut has_offline_required = false;
        let mut has_offline_optional = false;
        let mut has_degraded = false;

        for (_, state) in services.iter() {
            match state.state {
                HealthState::Offline => {
                    if state.is_required {
                        has_offline_required = true;
                    } else {
                        has_offline_optional = true;
                    }
                }
                HealthState::Degraded => {
                    has_degraded = true;
                }
                _ => {}
            }
        }

        drop(services);

        let new_mode = if has_offline_required {
            if self.is_degradation_enabled() {
                OperatingMode::Cached
            } else {
                OperatingMode::Offline
            }
        } else if has_offline_optional || has_degraded {
            OperatingMode::Degraded
        } else {
            OperatingMode::Normal
        };

        let mut mode = self.mode.write().unwrap();
        if *mode != new_mode {
            let old_mode = *mode;
            *mode = new_mode;
            info!(
                old_mode = ?old_mode,
                new_mode = ?new_mode,
                "Operating mode changed"
            );
        }
    }

    fn get_degraded_services(&self) -> Vec<String> {
        let services = self.services.read().unwrap();
        services
            .iter()
            .filter(|(_, s)| !matches!(s.state, HealthState::Healthy | HealthState::Unknown))
            .map(|(k, _)| k.to_string())
            .collect()
    }
}

impl Default for DegradationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Status information included in API responses when degraded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationStatus {
    /// Current operating mode
    pub mode: OperatingMode,
    /// Whether the system is currently degraded
    pub is_degraded: bool,
    /// Human-readable message
    pub message: String,
    /// List of degraded services
    pub degraded_services: Vec<String>,
    /// Number of requests served in degraded mode
    pub degraded_request_count: u64,
}

/// Wrapper for responses that may be cached or degraded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradedResponse<T> {
    /// The actual response data
    pub data: T,
    /// Whether this is a cached response
    pub is_cached: bool,
    /// Degradation status (only included if degraded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degradation: Option<DegradationStatus>,
    /// Cache age in seconds (only if cached)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_age_secs: Option<u64>,
}

impl<T> DegradedResponse<T> {
    /// Create a normal response
    pub fn normal(data: T) -> Self {
        Self {
            data,
            is_cached: false,
            degradation: None,
            cache_age_secs: None,
        }
    }

    /// Create a cached response
    pub fn cached(data: T, cache_age: Duration, status: DegradationStatus) -> Self {
        Self {
            data,
            is_cached: true,
            degradation: Some(status),
            cache_age_secs: Some(cache_age.as_secs()),
        }
    }

    /// Create a degraded (but live) response
    pub fn degraded(data: T, status: DegradationStatus) -> Self {
        Self {
            data,
            is_cached: false,
            degradation: Some(status),
            cache_age_secs: None,
        }
    }
}

/// Simple in-memory cache for degraded mode fallback
pub struct ResponseCache<T> {
    entries: RwLock<HashMap<String, CacheEntry<T>>>,
    max_entries: usize,
    ttl: Duration,
}

struct CacheEntry<T> {
    value: T,
    created_at: Instant,
}

impl<T: Clone> ResponseCache<T> {
    /// Create a new cache with specified capacity and TTL
    pub fn new(max_entries: usize, ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_entries,
            ttl,
        }
    }

    /// Store a value in the cache
    pub fn set(&self, key: impl Into<String>, value: T) {
        let key = key.into();
        let mut entries = self.entries.write().unwrap();

        // Evict expired entries if at capacity
        if entries.len() >= self.max_entries {
            let expired: Vec<_> = entries
                .iter()
                .filter(|(_, e)| e.created_at.elapsed() > self.ttl)
                .map(|(k, _)| k.clone())
                .collect();

            for k in expired {
                entries.remove(&k);
            }
        }

        entries.insert(
            key,
            CacheEntry {
                value,
                created_at: Instant::now(),
            },
        );
    }

    /// Get a value from the cache if it exists and is not expired
    pub fn get(&self, key: &str) -> Option<(T, Duration)> {
        let entries = self.entries.read().unwrap();
        entries.get(key).and_then(|entry| {
            let age = entry.created_at.elapsed();
            if age <= self.ttl {
                Some((entry.value.clone(), age))
            } else {
                None
            }
        })
    }

    /// Get a value even if expired (for degraded mode)
    pub fn get_stale(&self, key: &str) -> Option<(T, Duration)> {
        let entries = self.entries.read().unwrap();
        entries.get(key).map(|entry| {
            let age = entry.created_at.elapsed();
            (entry.value.clone(), age)
        })
    }

    /// Clear all entries
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
    }

    /// Get current number of entries
    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degradation_manager_modes() {
        let dm = DegradationManager::new();

        assert_eq!(dm.mode(), OperatingMode::Normal);
        assert!(dm.can_serve());

        // Record failures for optional service
        for _ in 0..5 {
            dm.record_failure(Service::Redis, "connection refused");
        }
        assert_eq!(dm.mode(), OperatingMode::Degraded);
        assert!(dm.can_serve());

        // Recover
        dm.record_success(Service::Redis);
        assert_eq!(dm.mode(), OperatingMode::Normal);
    }

    #[test]
    fn test_degradation_manager_required_service() {
        let dm = DegradationManager::new();

        // Fail required service
        for _ in 0..5 {
            dm.record_failure(Service::Neo4j, "connection refused");
        }

        // Should be cached mode (degradation enabled)
        assert_eq!(dm.mode(), OperatingMode::Cached);
        assert!(dm.can_serve()); // Still can serve from cache

        // Disable degradation
        dm.set_degradation_enabled(false);
        dm.record_failure(Service::Neo4j, "still failing");

        // Now should be offline
        assert_eq!(dm.mode(), OperatingMode::Offline);
        assert!(!dm.can_serve());
    }

    #[test]
    fn test_response_cache() {
        let cache = ResponseCache::new(10, Duration::from_secs(60));

        cache.set("key1", "value1".to_string());

        let (value, age) = cache.get("key1").unwrap();
        assert_eq!(value, "value1");
        assert!(age < Duration::from_secs(1));

        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_service_health() {
        let dm = DegradationManager::new();

        dm.record_success(Service::Qdrant);
        let health = dm.service_health(Service::Qdrant);
        assert_eq!(health.state, HealthState::Healthy);
        assert_eq!(health.failure_count, 0);

        dm.record_failure(Service::Qdrant, "error");
        dm.record_failure(Service::Qdrant, "error");
        dm.record_failure(Service::Qdrant, "error");

        let health = dm.service_health(Service::Qdrant);
        assert_eq!(health.state, HealthState::Degraded);
        assert_eq!(health.failure_count, 3);
    }
}
