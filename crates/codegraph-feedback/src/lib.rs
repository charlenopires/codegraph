//! CodeGraph Feedback - RLKGF (Reinforcement Learning from Knowledge Graph Feedback)
//!
//! This crate implements the feedback loop for improving code generation quality
//! through user feedback and confidence propagation in the knowledge graph.
//!
//! ## Components
//!
//! - [`FeedbackRepository`] - Persists feedback in PostgreSQL
//! - [`ConfidenceUpdater`] - Applies confidence deltas with clamping
//! - [`FeedbackType`] - Positive (thumbs up) or negative (thumbs down) feedback

pub mod confidence;
pub mod error;
pub mod metrics;
pub mod models;
pub mod prometheus;
pub mod propagation;
pub mod repository;
pub mod reward;

pub use confidence::{ConfidenceUpdate, ConfidenceUpdater, DEFAULT_CONFIDENCE, MAX_CONFIDENCE, MIN_CONFIDENCE};
pub use error::FeedbackError;
pub use metrics::{MetricsCollector, MetricsSnapshot, SharedMetricsCollector, new_shared_collector};
pub use models::{Feedback, FeedbackType};
pub use prometheus::{register_metrics, MetricsSummary};
pub use propagation::{
    ConfidencePropagator, PropagationRelation, PropagationResult, RelatedElement, RelationResolver,
    DEFAULT_DECAY_FACTOR, DEFAULT_MAX_HOPS,
};
pub use repository::FeedbackRepository;
pub use reward::{
    normalize_connectivity, RewardComputer, RewardResult, RewardSignals, RewardWeights,
    WEIGHT_BASE_CONFIDENCE, WEIGHT_CONNECTIVITY_BONUS, WEIGHT_NEGATIVE_PENALTY, WEIGHT_SIMILARITY_BONUS,
};
