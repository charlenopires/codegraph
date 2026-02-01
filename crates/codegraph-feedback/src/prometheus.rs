//! Prometheus metrics definitions for RLKGF feedback system
//!
//! This module defines all Prometheus metrics exposed by the feedback system.
//! Metrics are recorded using the `metrics` crate and can be exported via
//! `metrics-exporter-prometheus`.
//!
//! ## Counters
//!
//! - `feedback_total` - Total feedback events received
//! - `feedback_positive` - Positive feedback count (thumbs up)
//! - `feedback_negative` - Negative feedback count (thumbs down)
//! - `confidence_updates_total` - Total confidence updates applied
//! - `confidence_updates_positive` - Positive confidence updates
//! - `confidence_updates_negative` - Negative confidence updates
//! - `confidence_propagations_total` - Total propagation operations
//!
//! ## Gauges
//!
//! - `rlkgf_total_feedback` - Current total feedback count
//! - `rlkgf_positive_count` - Current positive feedback count
//! - `rlkgf_negative_count` - Current negative feedback count
//! - `rlkgf_positive_ratio` - Ratio of positive feedback (0.0-1.0)
//! - `rlkgf_negative_ratio` - Ratio of negative feedback (0.0-1.0)
//! - `rlkgf_average_confidence_delta` - Average confidence change
//! - `feedback_positive_ratio` - DB-derived positive ratio
//! - `feedback_negative_ratio` - DB-derived negative ratio
//! - `feedback_avg_confidence_delta` - DB-derived average delta
//!
//! ## Histograms
//!
//! - `reward_scores` - Distribution of computed reward scores
//! - `reward_base_component` - Base confidence component distribution
//! - `reward_similarity_component` - Similarity bonus distribution
//! - `reward_connectivity_component` - Connectivity bonus distribution
//! - `reward_penalty_component` - Negative penalty distribution
//! - `confidence_propagation_affected_count` - Elements affected per propagation
//! - `confidence_propagation_max_depth` - Max propagation depth reached

use metrics::{describe_counter, describe_gauge, describe_histogram, Unit};

// ==================== Counter Names ====================

/// Total feedback events received
pub const COUNTER_FEEDBACK_TOTAL: &str = "feedback_total";

/// Positive feedback count
pub const COUNTER_FEEDBACK_POSITIVE: &str = "feedback_positive";

/// Negative feedback count
pub const COUNTER_FEEDBACK_NEGATIVE: &str = "feedback_negative";

/// Total confidence updates applied
pub const COUNTER_CONFIDENCE_UPDATES_TOTAL: &str = "confidence_updates_total";

/// Positive confidence updates
pub const COUNTER_CONFIDENCE_UPDATES_POSITIVE: &str = "confidence_updates_positive";

/// Negative confidence updates
pub const COUNTER_CONFIDENCE_UPDATES_NEGATIVE: &str = "confidence_updates_negative";

/// Total propagation operations
pub const COUNTER_PROPAGATIONS_TOTAL: &str = "confidence_propagations_total";

// ==================== Gauge Names ====================

/// Current total feedback (in-memory)
pub const GAUGE_RLKGF_TOTAL_FEEDBACK: &str = "rlkgf_total_feedback";

/// Current positive count (in-memory)
pub const GAUGE_RLKGF_POSITIVE_COUNT: &str = "rlkgf_positive_count";

/// Current negative count (in-memory)
pub const GAUGE_RLKGF_NEGATIVE_COUNT: &str = "rlkgf_negative_count";

/// Positive ratio (in-memory)
pub const GAUGE_RLKGF_POSITIVE_RATIO: &str = "rlkgf_positive_ratio";

/// Negative ratio (in-memory)
pub const GAUGE_RLKGF_NEGATIVE_RATIO: &str = "rlkgf_negative_ratio";

/// Average confidence delta (in-memory)
pub const GAUGE_RLKGF_AVG_DELTA: &str = "rlkgf_average_confidence_delta";

/// Positive ratio (from database)
pub const GAUGE_FEEDBACK_POSITIVE_RATIO: &str = "feedback_positive_ratio";

/// Negative ratio (from database)
pub const GAUGE_FEEDBACK_NEGATIVE_RATIO: &str = "feedback_negative_ratio";

/// Average confidence delta (from database)
pub const GAUGE_FEEDBACK_AVG_DELTA: &str = "feedback_avg_confidence_delta";

// ==================== Histogram Names ====================

/// Reward score distribution
pub const HISTOGRAM_REWARD_SCORES: &str = "reward_scores";

/// Base confidence component distribution
pub const HISTOGRAM_REWARD_BASE: &str = "reward_base_component";

/// Similarity bonus component distribution
pub const HISTOGRAM_REWARD_SIMILARITY: &str = "reward_similarity_component";

/// Connectivity bonus component distribution
pub const HISTOGRAM_REWARD_CONNECTIVITY: &str = "reward_connectivity_component";

/// Penalty component distribution
pub const HISTOGRAM_REWARD_PENALTY: &str = "reward_penalty_component";

/// Elements affected per propagation
pub const HISTOGRAM_PROPAGATION_AFFECTED: &str = "confidence_propagation_affected_count";

/// Max depth reached per propagation
pub const HISTOGRAM_PROPAGATION_DEPTH: &str = "confidence_propagation_max_depth";

/// Register all RLKGF metrics with descriptions
///
/// Call this function during application startup to register metric descriptions
/// with the metrics registry. This enables proper Prometheus exposition.
///
/// # Example
///
/// ```ignore
/// use codegraph_feedback::prometheus::register_metrics;
///
/// fn main() {
///     // Initialize your metrics exporter first
///     // e.g., metrics_exporter_prometheus::PrometheusBuilder::new().install()
///
///     // Then register RLKGF metrics
///     register_metrics();
/// }
/// ```
pub fn register_metrics() {
    // Counters
    describe_counter!(
        COUNTER_FEEDBACK_TOTAL,
        Unit::Count,
        "Total number of feedback events received"
    );
    describe_counter!(
        COUNTER_FEEDBACK_POSITIVE,
        Unit::Count,
        "Number of positive (thumbs up) feedback events"
    );
    describe_counter!(
        COUNTER_FEEDBACK_NEGATIVE,
        Unit::Count,
        "Number of negative (thumbs down) feedback events"
    );
    describe_counter!(
        COUNTER_CONFIDENCE_UPDATES_TOTAL,
        Unit::Count,
        "Total confidence updates applied to elements"
    );
    describe_counter!(
        COUNTER_CONFIDENCE_UPDATES_POSITIVE,
        Unit::Count,
        "Confidence updates from positive feedback"
    );
    describe_counter!(
        COUNTER_CONFIDENCE_UPDATES_NEGATIVE,
        Unit::Count,
        "Confidence updates from negative feedback"
    );
    describe_counter!(
        COUNTER_PROPAGATIONS_TOTAL,
        Unit::Count,
        "Total confidence propagation operations"
    );

    // Gauges
    describe_gauge!(
        GAUGE_RLKGF_TOTAL_FEEDBACK,
        Unit::Count,
        "Current total feedback count (in-memory)"
    );
    describe_gauge!(
        GAUGE_RLKGF_POSITIVE_COUNT,
        Unit::Count,
        "Current positive feedback count (in-memory)"
    );
    describe_gauge!(
        GAUGE_RLKGF_NEGATIVE_COUNT,
        Unit::Count,
        "Current negative feedback count (in-memory)"
    );
    describe_gauge!(
        GAUGE_RLKGF_POSITIVE_RATIO,
        Unit::Percent,
        "Ratio of positive feedback (0.0-1.0)"
    );
    describe_gauge!(
        GAUGE_RLKGF_NEGATIVE_RATIO,
        Unit::Percent,
        "Ratio of negative feedback (0.0-1.0)"
    );
    describe_gauge!(
        GAUGE_RLKGF_AVG_DELTA,
        "Average confidence delta per feedback"
    );
    describe_gauge!(
        GAUGE_FEEDBACK_POSITIVE_RATIO,
        Unit::Percent,
        "Ratio of positive feedback from database"
    );
    describe_gauge!(
        GAUGE_FEEDBACK_NEGATIVE_RATIO,
        Unit::Percent,
        "Ratio of negative feedback from database"
    );
    describe_gauge!(
        GAUGE_FEEDBACK_AVG_DELTA,
        "Average confidence delta from database"
    );

    // Histograms
    describe_histogram!(
        HISTOGRAM_REWARD_SCORES,
        "Distribution of computed reward scores"
    );
    describe_histogram!(
        HISTOGRAM_REWARD_BASE,
        "Distribution of base confidence component in rewards"
    );
    describe_histogram!(
        HISTOGRAM_REWARD_SIMILARITY,
        "Distribution of similarity bonus component in rewards"
    );
    describe_histogram!(
        HISTOGRAM_REWARD_CONNECTIVITY,
        "Distribution of connectivity bonus component in rewards"
    );
    describe_histogram!(
        HISTOGRAM_REWARD_PENALTY,
        "Distribution of negative penalty component in rewards"
    );
    describe_histogram!(
        HISTOGRAM_PROPAGATION_AFFECTED,
        Unit::Count,
        "Number of elements affected per propagation operation"
    );
    describe_histogram!(
        HISTOGRAM_PROPAGATION_DEPTH,
        Unit::Count,
        "Maximum hop depth reached per propagation"
    );
}

/// Summary of all exposed metrics for documentation
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    /// Counter metric names
    pub counters: Vec<&'static str>,
    /// Gauge metric names
    pub gauges: Vec<&'static str>,
    /// Histogram metric names
    pub histograms: Vec<&'static str>,
}

impl MetricsSummary {
    /// Get a summary of all RLKGF metrics
    pub fn all() -> Self {
        Self {
            counters: vec![
                COUNTER_FEEDBACK_TOTAL,
                COUNTER_FEEDBACK_POSITIVE,
                COUNTER_FEEDBACK_NEGATIVE,
                COUNTER_CONFIDENCE_UPDATES_TOTAL,
                COUNTER_CONFIDENCE_UPDATES_POSITIVE,
                COUNTER_CONFIDENCE_UPDATES_NEGATIVE,
                COUNTER_PROPAGATIONS_TOTAL,
            ],
            gauges: vec![
                GAUGE_RLKGF_TOTAL_FEEDBACK,
                GAUGE_RLKGF_POSITIVE_COUNT,
                GAUGE_RLKGF_NEGATIVE_COUNT,
                GAUGE_RLKGF_POSITIVE_RATIO,
                GAUGE_RLKGF_NEGATIVE_RATIO,
                GAUGE_RLKGF_AVG_DELTA,
                GAUGE_FEEDBACK_POSITIVE_RATIO,
                GAUGE_FEEDBACK_NEGATIVE_RATIO,
                GAUGE_FEEDBACK_AVG_DELTA,
            ],
            histograms: vec![
                HISTOGRAM_REWARD_SCORES,
                HISTOGRAM_REWARD_BASE,
                HISTOGRAM_REWARD_SIMILARITY,
                HISTOGRAM_REWARD_CONNECTIVITY,
                HISTOGRAM_REWARD_PENALTY,
                HISTOGRAM_PROPAGATION_AFFECTED,
                HISTOGRAM_PROPAGATION_DEPTH,
            ],
        }
    }

    /// Total number of metrics exposed
    pub fn total_metrics(&self) -> usize {
        self.counters.len() + self.gauges.len() + self.histograms.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_metrics_does_not_panic() {
        // Should not panic even without a recorder installed
        register_metrics();
    }

    #[test]
    fn test_metrics_summary() {
        let summary = MetricsSummary::all();

        assert_eq!(summary.counters.len(), 7);
        assert_eq!(summary.gauges.len(), 9);
        assert_eq!(summary.histograms.len(), 7);
        assert_eq!(summary.total_metrics(), 23);
    }

    #[test]
    fn test_metric_names_are_valid() {
        let summary = MetricsSummary::all();

        // All metric names should be non-empty and contain only valid characters
        for name in summary.counters.iter()
            .chain(summary.gauges.iter())
            .chain(summary.histograms.iter())
        {
            assert!(!name.is_empty());
            assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        }
    }
}
