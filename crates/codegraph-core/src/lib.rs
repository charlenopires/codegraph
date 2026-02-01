//! CodeGraph Core - Domain entities and traits
pub mod config;
pub mod degradation;
pub mod entities;
pub mod error;
pub mod error_tracking;
pub mod retry;
pub mod traits;

pub use config::Config;
pub use degradation::{
    DegradationManager, DegradationStatus, DegradedResponse, HealthState, OperatingMode,
    ResponseCache, Service, ServiceHealth,
};
pub use entities::*;
pub use error::*;
pub use error_tracking::{
    capture_error, capture_error_with_context, capture_message, install_panic_hook,
    ErrorContext, ErrorTracker, ErrorTrackerConfig, MessageLevel,
};
pub use retry::{
    with_retry, with_retry_and_circuit_breaker, CircuitBreaker, CircuitOpenError, CircuitState,
    RetryError, RetryMetrics, RetryPolicy, RetryResult, ServiceType,
};
pub use traits::*;
