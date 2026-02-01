//! Error types for the feedback module

use thiserror::Error;

/// Feedback module errors
#[derive(Debug, Error)]
pub enum FeedbackError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Feedback not found
    #[error("Feedback not found: {0}")]
    NotFound(uuid::Uuid),

    /// Element not found
    #[error("Element not found: {0}")]
    ElementNotFound(uuid::Uuid),

    /// Invalid feedback type
    #[error("Invalid feedback type: {0}")]
    InvalidFeedbackType(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for feedback operations
pub type Result<T> = std::result::Result<T, FeedbackError>;
