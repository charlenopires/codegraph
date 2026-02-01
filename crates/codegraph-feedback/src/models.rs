//! Feedback data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Feedback type - positive or negative user signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "feedback_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    /// User liked the generation
    ThumbsUp,
    /// User disliked the generation
    ThumbsDown,
}

impl FeedbackType {
    /// Returns the confidence delta for this feedback type
    ///
    /// - ThumbsUp: +0.1
    /// - ThumbsDown: -0.15
    pub fn confidence_delta(&self) -> f32 {
        match self {
            FeedbackType::ThumbsUp => 0.1,
            FeedbackType::ThumbsDown => -0.15,
        }
    }

    /// Returns true if this is positive feedback
    pub fn is_positive(&self) -> bool {
        matches!(self, FeedbackType::ThumbsUp)
    }
}

impl std::fmt::Display for FeedbackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedbackType::ThumbsUp => write!(f, "thumbs_up"),
            FeedbackType::ThumbsDown => write!(f, "thumbs_down"),
        }
    }
}

/// Feedback record stored in PostgreSQL
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Feedback {
    /// Unique feedback ID
    pub id: Uuid,

    /// ID of the generation that received feedback
    pub generation_id: Uuid,

    /// IDs of elements referenced in the generation (stored as JSON array)
    #[sqlx(json)]
    pub element_ids: Vec<Uuid>,

    /// Type of feedback (positive/negative)
    pub feedback_type: FeedbackType,

    /// Optional query context for the generation
    pub query_context: Option<String>,

    /// Optional user comment
    pub comment: Option<String>,

    /// Confidence delta applied to elements
    pub confidence_delta: f32,

    /// When the feedback was submitted
    pub created_at: DateTime<Utc>,
}

/// Request to create new feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFeedback {
    /// ID of the generation that received feedback
    pub generation_id: Uuid,

    /// IDs of elements referenced in the generation
    pub element_ids: Vec<Uuid>,

    /// Type of feedback (positive/negative)
    pub feedback_type: FeedbackType,

    /// Optional query context for the generation
    pub query_context: Option<String>,

    /// Optional user comment
    pub comment: Option<String>,
}

/// Summary of feedback for an element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSummary {
    /// Element ID
    pub element_id: Uuid,

    /// Total positive feedback count
    pub positive_count: i64,

    /// Total negative feedback count
    pub negative_count: i64,

    /// Net confidence delta applied
    pub net_confidence_delta: f32,

    /// Last feedback timestamp
    pub last_feedback_at: Option<DateTime<Utc>>,
}

/// Aggregated feedback metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeedbackMetrics {
    /// Total feedback count
    pub total_feedback: i64,

    /// Positive feedback count
    pub positive_count: i64,

    /// Negative feedback count
    pub negative_count: i64,

    /// Positive ratio (0.0 - 1.0)
    pub positive_ratio: f64,

    /// Negative ratio (0.0 - 1.0)
    pub negative_ratio: f64,

    /// Average confidence delta
    pub avg_confidence_delta: f64,
}
