//! MetricsCollector - Aggregates and exposes RLKGF feedback metrics
//!
//! Collects and exposes the following metrics:
//! - `total_feedback`: Total number of feedback events
//! - `positive_ratio`: Ratio of positive feedback (0.0 - 1.0)
//! - `negative_ratio`: Ratio of negative feedback (0.0 - 1.0)
//! - `average_confidence_delta`: Average confidence change per feedback

use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, info};

use crate::models::FeedbackType;

/// Thread-safe metrics collector for RLKGF feedback
#[derive(Debug)]
pub struct MetricsCollector {
    /// Total feedback count
    total_feedback: AtomicU64,
    /// Positive feedback count
    positive_count: AtomicU64,
    /// Negative feedback count
    negative_count: AtomicU64,
    /// Sum of confidence deltas (stored as fixed-point: value * 10000)
    confidence_delta_sum: AtomicI64,
    /// Count for confidence delta average calculation
    confidence_delta_count: AtomicU64,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create a new MetricsCollector
    pub fn new() -> Self {
        Self {
            total_feedback: AtomicU64::new(0),
            positive_count: AtomicU64::new(0),
            negative_count: AtomicU64::new(0),
            confidence_delta_sum: AtomicI64::new(0),
            confidence_delta_count: AtomicU64::new(0),
        }
    }

    /// Record a feedback event
    pub fn record_feedback(&self, feedback_type: FeedbackType) {
        self.total_feedback.fetch_add(1, Ordering::Relaxed);

        let delta = feedback_type.confidence_delta();

        if feedback_type.is_positive() {
            self.positive_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.negative_count.fetch_add(1, Ordering::Relaxed);
        }

        // Store delta as fixed-point integer (multiply by 10000 for precision)
        let delta_fixed = (delta * 10000.0) as i64;
        self.confidence_delta_sum.fetch_add(delta_fixed, Ordering::Relaxed);
        self.confidence_delta_count.fetch_add(1, Ordering::Relaxed);

        debug!(
            feedback_type = %feedback_type,
            delta = delta,
            total = self.total_feedback.load(Ordering::Relaxed),
            "Recorded feedback"
        );
    }

    /// Record a custom confidence delta (for propagated updates)
    pub fn record_confidence_delta(&self, delta: f32) {
        let delta_fixed = (delta * 10000.0) as i64;
        self.confidence_delta_sum.fetch_add(delta_fixed, Ordering::Relaxed);
        self.confidence_delta_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get total feedback count
    pub fn total_feedback(&self) -> u64 {
        self.total_feedback.load(Ordering::Relaxed)
    }

    /// Get positive feedback count
    pub fn positive_count(&self) -> u64 {
        self.positive_count.load(Ordering::Relaxed)
    }

    /// Get negative feedback count
    pub fn negative_count(&self) -> u64 {
        self.negative_count.load(Ordering::Relaxed)
    }

    /// Get positive feedback ratio (0.0 - 1.0)
    pub fn positive_ratio(&self) -> f64 {
        let total = self.total_feedback.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        self.positive_count.load(Ordering::Relaxed) as f64 / total as f64
    }

    /// Get negative feedback ratio (0.0 - 1.0)
    pub fn negative_ratio(&self) -> f64 {
        let total = self.total_feedback.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        self.negative_count.load(Ordering::Relaxed) as f64 / total as f64
    }

    /// Get average confidence delta
    pub fn average_confidence_delta(&self) -> f64 {
        let count = self.confidence_delta_count.load(Ordering::Relaxed);
        if count == 0 {
            return 0.0;
        }
        let sum = self.confidence_delta_sum.load(Ordering::Relaxed);
        (sum as f64 / 10000.0) / count as f64
    }

    /// Get a snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_feedback: self.total_feedback(),
            positive_count: self.positive_count(),
            negative_count: self.negative_count(),
            positive_ratio: self.positive_ratio(),
            negative_ratio: self.negative_ratio(),
            average_confidence_delta: self.average_confidence_delta(),
        }
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.total_feedback.store(0, Ordering::Relaxed);
        self.positive_count.store(0, Ordering::Relaxed);
        self.negative_count.store(0, Ordering::Relaxed);
        self.confidence_delta_sum.store(0, Ordering::Relaxed);
        self.confidence_delta_count.store(0, Ordering::Relaxed);

        info!("Metrics reset");
    }

    /// Export metrics to Prometheus format
    ///
    /// This updates the Prometheus metrics registry with current values.
    pub fn export_to_prometheus(&self) {
        let snapshot = self.snapshot();

        metrics::gauge!("rlkgf_total_feedback").set(snapshot.total_feedback as f64);
        metrics::gauge!("rlkgf_positive_count").set(snapshot.positive_count as f64);
        metrics::gauge!("rlkgf_negative_count").set(snapshot.negative_count as f64);
        metrics::gauge!("rlkgf_positive_ratio").set(snapshot.positive_ratio);
        metrics::gauge!("rlkgf_negative_ratio").set(snapshot.negative_ratio);
        metrics::gauge!("rlkgf_average_confidence_delta").set(snapshot.average_confidence_delta);

        debug!(
            total = snapshot.total_feedback,
            positive_ratio = snapshot.positive_ratio,
            negative_ratio = snapshot.negative_ratio,
            avg_delta = snapshot.average_confidence_delta,
            "Exported metrics to Prometheus"
        );
    }
}

/// Point-in-time snapshot of all metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricsSnapshot {
    /// Total number of feedback events
    pub total_feedback: u64,
    /// Number of positive feedback events
    pub positive_count: u64,
    /// Number of negative feedback events
    pub negative_count: u64,
    /// Ratio of positive feedback (0.0 - 1.0)
    pub positive_ratio: f64,
    /// Ratio of negative feedback (0.0 - 1.0)
    pub negative_ratio: f64,
    /// Average confidence delta per feedback
    pub average_confidence_delta: f64,
}

impl MetricsSnapshot {
    /// Returns true if feedback is predominantly positive (> 60%)
    pub fn is_positive_trending(&self) -> bool {
        self.positive_ratio > 0.6
    }

    /// Returns true if feedback is predominantly negative (> 60%)
    pub fn is_negative_trending(&self) -> bool {
        self.negative_ratio > 0.6
    }

    /// Returns true if feedback is balanced (40-60% positive)
    pub fn is_balanced(&self) -> bool {
        self.positive_ratio >= 0.4 && self.positive_ratio <= 0.6
    }

    /// Returns the net confidence impact (sum of deltas)
    pub fn net_confidence_impact(&self) -> f64 {
        self.average_confidence_delta * self.total_feedback as f64
    }
}

/// Shared metrics collector that can be cloned across threads
pub type SharedMetricsCollector = Arc<MetricsCollector>;

/// Create a new shared metrics collector
pub fn new_shared_collector() -> SharedMetricsCollector {
    Arc::new(MetricsCollector::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collector_is_empty() {
        let collector = MetricsCollector::new();

        assert_eq!(collector.total_feedback(), 0);
        assert_eq!(collector.positive_count(), 0);
        assert_eq!(collector.negative_count(), 0);
        assert_eq!(collector.positive_ratio(), 0.0);
        assert_eq!(collector.negative_ratio(), 0.0);
        assert_eq!(collector.average_confidence_delta(), 0.0);
    }

    #[test]
    fn test_record_positive_feedback() {
        let collector = MetricsCollector::new();

        collector.record_feedback(FeedbackType::ThumbsUp);

        assert_eq!(collector.total_feedback(), 1);
        assert_eq!(collector.positive_count(), 1);
        assert_eq!(collector.negative_count(), 0);
        assert_eq!(collector.positive_ratio(), 1.0);
        assert_eq!(collector.negative_ratio(), 0.0);
        assert!((collector.average_confidence_delta() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_record_negative_feedback() {
        let collector = MetricsCollector::new();

        collector.record_feedback(FeedbackType::ThumbsDown);

        assert_eq!(collector.total_feedback(), 1);
        assert_eq!(collector.positive_count(), 0);
        assert_eq!(collector.negative_count(), 1);
        assert_eq!(collector.positive_ratio(), 0.0);
        assert_eq!(collector.negative_ratio(), 1.0);
        assert!((collector.average_confidence_delta() - (-0.15)).abs() < 0.001);
    }

    #[test]
    fn test_mixed_feedback() {
        let collector = MetricsCollector::new();

        collector.record_feedback(FeedbackType::ThumbsUp);
        collector.record_feedback(FeedbackType::ThumbsUp);
        collector.record_feedback(FeedbackType::ThumbsDown);

        assert_eq!(collector.total_feedback(), 3);
        assert_eq!(collector.positive_count(), 2);
        assert_eq!(collector.negative_count(), 1);
        assert!((collector.positive_ratio() - 0.6667).abs() < 0.01);
        assert!((collector.negative_ratio() - 0.3333).abs() < 0.01);

        // Average: (0.1 + 0.1 - 0.15) / 3 = 0.05 / 3 ≈ 0.0167
        assert!((collector.average_confidence_delta() - 0.0167).abs() < 0.01);
    }

    #[test]
    fn test_snapshot() {
        let collector = MetricsCollector::new();

        collector.record_feedback(FeedbackType::ThumbsUp);
        collector.record_feedback(FeedbackType::ThumbsDown);

        let snapshot = collector.snapshot();

        assert_eq!(snapshot.total_feedback, 2);
        assert_eq!(snapshot.positive_count, 1);
        assert_eq!(snapshot.negative_count, 1);
        assert_eq!(snapshot.positive_ratio, 0.5);
        assert_eq!(snapshot.negative_ratio, 0.5);
    }

    #[test]
    fn test_reset() {
        let collector = MetricsCollector::new();

        collector.record_feedback(FeedbackType::ThumbsUp);
        collector.record_feedback(FeedbackType::ThumbsDown);

        collector.reset();

        assert_eq!(collector.total_feedback(), 0);
        assert_eq!(collector.positive_count(), 0);
        assert_eq!(collector.negative_count(), 0);
    }

    #[test]
    fn test_snapshot_trending() {
        let collector = MetricsCollector::new();

        // Predominantly positive
        for _ in 0..7 {
            collector.record_feedback(FeedbackType::ThumbsUp);
        }
        for _ in 0..3 {
            collector.record_feedback(FeedbackType::ThumbsDown);
        }

        let snapshot = collector.snapshot();
        assert!(snapshot.is_positive_trending());
        assert!(!snapshot.is_negative_trending());
        assert!(!snapshot.is_balanced());
    }

    #[test]
    fn test_snapshot_balanced() {
        let collector = MetricsCollector::new();

        for _ in 0..5 {
            collector.record_feedback(FeedbackType::ThumbsUp);
        }
        for _ in 0..5 {
            collector.record_feedback(FeedbackType::ThumbsDown);
        }

        let snapshot = collector.snapshot();
        assert!(snapshot.is_balanced());
        assert!(!snapshot.is_positive_trending());
        assert!(!snapshot.is_negative_trending());
    }

    #[test]
    fn test_record_custom_delta() {
        let collector = MetricsCollector::new();

        collector.record_confidence_delta(0.05);
        collector.record_confidence_delta(0.025);

        // Average: (0.05 + 0.025) / 2 = 0.0375
        assert!((collector.average_confidence_delta() - 0.0375).abs() < 0.001);
    }

    #[test]
    fn test_shared_collector() {
        let collector = new_shared_collector();
        let collector2 = Arc::clone(&collector);

        collector.record_feedback(FeedbackType::ThumbsUp);
        collector2.record_feedback(FeedbackType::ThumbsDown);

        assert_eq!(collector.total_feedback(), 2);
        assert_eq!(collector2.total_feedback(), 2);
    }

    #[test]
    fn test_net_confidence_impact() {
        let collector = MetricsCollector::new();

        collector.record_feedback(FeedbackType::ThumbsUp); // +0.1
        collector.record_feedback(FeedbackType::ThumbsUp); // +0.1
        collector.record_feedback(FeedbackType::ThumbsDown); // -0.15

        let snapshot = collector.snapshot();

        // Net: 0.1 + 0.1 - 0.15 = 0.05
        assert!((snapshot.net_confidence_impact() - 0.05).abs() < 0.01);
    }
}
