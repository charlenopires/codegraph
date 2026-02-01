//! Benchmark error types

use thiserror::Error;

/// Benchmark-specific errors
#[derive(Debug, Error)]
pub enum BenchmarkError {
    /// Vector storage error
    #[error("Vector error: {0}")]
    Vector(#[from] codegraph_vector::VectorError),

    /// Query error
    #[error("Query failed: {0}")]
    QueryFailed(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Dataset error
    #[error("Dataset error: {0}")]
    Dataset(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Metrics calculation error
    #[error("Metrics error: {0}")]
    Metrics(String),
}

/// Result type for benchmark operations
pub type Result<T> = std::result::Result<T, BenchmarkError>;
