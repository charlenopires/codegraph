//! Error types for the vector module

use thiserror::Error;

/// Vector module errors
#[derive(Debug, Error)]
pub enum VectorError {
    /// Qdrant client error
    #[error("Qdrant error: {0}")]
    Qdrant(#[from] qdrant_client::QdrantError),

    /// Collection not found
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    /// Point not found
    #[error("Point not found: {0}")]
    PointNotFound(String),

    /// Invalid vector dimension
    #[error("Invalid vector dimension: expected {expected}, got {actual}")]
    InvalidDimension { expected: usize, actual: usize },

    /// Redis cache error
    #[error("Cache error: {0}")]
    Cache(#[from] redis::RedisError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),
}

/// Result type for vector operations
pub type Result<T> = std::result::Result<T, VectorError>;
