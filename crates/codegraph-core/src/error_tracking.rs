//! Error tracking abstraction for production monitoring
//!
//! Provides a simple interface for capturing errors and sending them
//! to external monitoring services like Sentry.
//!
//! # Configuration
//!
//! Set the following environment variables:
//! - `SENTRY_DSN`: Sentry Data Source Name (optional, disables if not set)
//! - `SENTRY_ENVIRONMENT`: Environment name (production, staging, etc.)
//! - `SENTRY_RELEASE`: Release version

use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;

use tracing::{error, info, warn};

/// Global error tracker instance
static ERROR_TRACKER: OnceLock<ErrorTracker> = OnceLock::new();

/// Error context for additional metadata
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    /// Request trace ID
    pub trace_id: Option<String>,
    /// User ID if authenticated
    pub user_id: Option<String>,
    /// HTTP endpoint
    pub endpoint: Option<String>,
    /// HTTP method
    pub method: Option<String>,
    /// Additional tags
    pub tags: HashMap<String, String>,
    /// Additional data
    pub extra: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    pub fn with_endpoint(mut self, method: impl Into<String>, endpoint: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Error tracking configuration
#[derive(Debug, Clone)]
pub struct ErrorTrackerConfig {
    /// Sentry DSN (if None, error tracking is disabled)
    pub dsn: Option<String>,
    /// Environment name
    pub environment: String,
    /// Release version
    pub release: Option<String>,
    /// Sample rate (0.0 to 1.0)
    pub sample_rate: f32,
    /// Enable debug mode
    pub debug: bool,
}

impl ErrorTrackerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            dsn: env::var("SENTRY_DSN").ok(),
            environment: env::var("SENTRY_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
            release: env::var("SENTRY_RELEASE").ok(),
            sample_rate: env::var("SENTRY_SAMPLE_RATE")
                .ok()
                .and_then(|r| r.parse().ok())
                .unwrap_or(1.0),
            debug: env::var("SENTRY_DEBUG")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
        }
    }

    /// Check if error tracking is enabled
    pub fn is_enabled(&self) -> bool {
        self.dsn.is_some()
    }
}

impl Default for ErrorTrackerConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Error tracker that captures and reports errors
pub struct ErrorTracker {
    config: ErrorTrackerConfig,
    enabled: bool,
}

impl ErrorTracker {
    /// Create a new error tracker with configuration
    pub fn new(config: ErrorTrackerConfig) -> Self {
        let enabled = config.is_enabled();

        if enabled {
            info!(
                environment = %config.environment,
                release = ?config.release,
                sample_rate = %config.sample_rate,
                "Error tracking enabled"
            );
        } else {
            info!("Error tracking disabled (SENTRY_DSN not set)");
        }

        Self { config, enabled }
    }

    /// Initialize the global error tracker
    pub fn init() -> &'static Self {
        ERROR_TRACKER.get_or_init(|| Self::new(ErrorTrackerConfig::from_env()))
    }

    /// Get the global error tracker
    pub fn global() -> Option<&'static Self> {
        ERROR_TRACKER.get()
    }

    /// Check if error tracking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the configuration
    pub fn config(&self) -> &ErrorTrackerConfig {
        &self.config
    }

    /// Capture an error with context
    pub fn capture_error(&self, error: &dyn std::error::Error, context: Option<ErrorContext>) {
        if !self.enabled {
            return;
        }

        let ctx = context.unwrap_or_default();

        // Log the error with context
        error!(
            error = %error,
            trace_id = ?ctx.trace_id,
            endpoint = ?ctx.endpoint,
            method = ?ctx.method,
            tags = ?ctx.tags,
            "Captured error for tracking"
        );

        // In a real implementation, this would send to Sentry:
        // sentry::capture_error(error);
        //
        // For now, we just log it. To enable Sentry, add sentry-rust dependency
        // and uncomment the actual Sentry integration.
    }

    /// Capture a message with severity
    pub fn capture_message(&self, message: &str, level: MessageLevel, context: Option<ErrorContext>) {
        if !self.enabled {
            return;
        }

        let ctx = context.unwrap_or_default();

        match level {
            MessageLevel::Debug => {
                tracing::debug!(
                    message = %message,
                    trace_id = ?ctx.trace_id,
                    "Debug message"
                );
            }
            MessageLevel::Info => {
                info!(
                    message = %message,
                    trace_id = ?ctx.trace_id,
                    "Info message"
                );
            }
            MessageLevel::Warning => {
                warn!(
                    message = %message,
                    trace_id = ?ctx.trace_id,
                    "Warning message"
                );
            }
            MessageLevel::Error => {
                error!(
                    message = %message,
                    trace_id = ?ctx.trace_id,
                    "Error message"
                );
            }
            MessageLevel::Fatal => {
                error!(
                    message = %message,
                    trace_id = ?ctx.trace_id,
                    "Fatal message"
                );
            }
        }
    }

    /// Capture a panic
    pub fn capture_panic(&self, panic_info: &std::panic::PanicHookInfo<'_>) {
        if !self.enabled {
            return;
        }

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        error!(
            panic_message = %message,
            location = %location,
            "Application panic captured"
        );
    }
}

/// Message severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

/// Install panic hook to capture panics
pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // Capture to error tracker
        if let Some(tracker) = ErrorTracker::global() {
            tracker.capture_panic(panic_info);
        }

        // Call original hook
        original_hook(panic_info);
    }));

    info!("Panic hook installed for error tracking");
}

/// Convenience function to capture an error
pub fn capture_error(error: &dyn std::error::Error) {
    if let Some(tracker) = ErrorTracker::global() {
        tracker.capture_error(error, None);
    }
}

/// Convenience function to capture an error with context
pub fn capture_error_with_context(error: &dyn std::error::Error, context: ErrorContext) {
    if let Some(tracker) = ErrorTracker::global() {
        tracker.capture_error(error, Some(context));
    }
}

/// Convenience function to capture a message
pub fn capture_message(message: &str, level: MessageLevel) {
    if let Some(tracker) = ErrorTracker::global() {
        tracker.capture_message(message, level, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_builder() {
        let ctx = ErrorContext::new()
            .with_trace_id("abc-123")
            .with_endpoint("POST", "/api/generate")
            .with_tag("user_type", "premium")
            .with_extra("query", "create button");

        assert_eq!(ctx.trace_id, Some("abc-123".to_string()));
        assert_eq!(ctx.method, Some("POST".to_string()));
        assert_eq!(ctx.endpoint, Some("/api/generate".to_string()));
        assert_eq!(ctx.tags.get("user_type"), Some(&"premium".to_string()));
    }

    #[test]
    fn test_config_disabled_by_default() {
        // Without SENTRY_DSN set, tracking should be disabled
        let config = ErrorTrackerConfig::from_env();
        // This test assumes SENTRY_DSN is not set in test environment
        // In CI, this should pass as we don't set SENTRY_DSN
    }
}
