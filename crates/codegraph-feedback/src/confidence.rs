//! ConfidenceUpdater - Applies confidence deltas to elements
//!
//! Implements RLKGF confidence updates based on user feedback:
//! - ThumbsUp: +0.1 delta
//! - ThumbsDown: -0.15 delta
//! - Values clamped between 0.1 and 0.99

use tracing::{debug, instrument};
use uuid::Uuid;

use crate::models::FeedbackType;

/// Minimum confidence value (prevents complete distrust)
pub const MIN_CONFIDENCE: f32 = 0.1;

/// Maximum confidence value (prevents complete certainty)
pub const MAX_CONFIDENCE: f32 = 0.99;

/// Default confidence for new elements
pub const DEFAULT_CONFIDENCE: f32 = 0.5;

/// Confidence update result
#[derive(Debug, Clone, Copy)]
pub struct ConfidenceUpdate {
    /// Element ID that was updated
    pub element_id: Uuid,
    /// Original confidence value
    pub old_confidence: f32,
    /// New confidence value after applying delta
    pub new_confidence: f32,
    /// Delta that was applied
    pub delta: f32,
}

impl ConfidenceUpdate {
    /// Returns the actual change in confidence (may differ from delta due to clamping)
    pub fn actual_delta(&self) -> f32 {
        self.new_confidence - self.old_confidence
    }

    /// Returns true if the confidence increased
    pub fn is_increase(&self) -> bool {
        self.new_confidence > self.old_confidence
    }
}

/// Updates element confidence values based on feedback
#[derive(Debug, Clone, Default)]
pub struct ConfidenceUpdater {
    /// Minimum allowed confidence
    min_confidence: f32,
    /// Maximum allowed confidence
    max_confidence: f32,
}

impl ConfidenceUpdater {
    /// Create a new ConfidenceUpdater with default bounds (0.1 - 0.99)
    pub fn new() -> Self {
        Self {
            min_confidence: MIN_CONFIDENCE,
            max_confidence: MAX_CONFIDENCE,
        }
    }

    /// Create a ConfidenceUpdater with custom bounds
    pub fn with_bounds(min: f32, max: f32) -> Self {
        assert!(min >= 0.0 && min < max && max <= 1.0, "Invalid bounds");
        Self {
            min_confidence: min,
            max_confidence: max,
        }
    }

    /// Calculate the new confidence value after applying a feedback delta
    ///
    /// The delta is determined by the feedback type:
    /// - ThumbsUp: +0.1
    /// - ThumbsDown: -0.15
    ///
    /// The result is clamped between min_confidence and max_confidence.
    #[instrument(skip(self))]
    pub fn calculate_update(
        &self,
        element_id: Uuid,
        current_confidence: f32,
        feedback_type: FeedbackType,
    ) -> ConfidenceUpdate {
        let delta = feedback_type.confidence_delta();
        let unclamped = current_confidence + delta;
        let new_confidence = self.clamp(unclamped);

        debug!(
            element_id = %element_id,
            current = current_confidence,
            delta = delta,
            new = new_confidence,
            feedback = %feedback_type,
            "Calculated confidence update"
        );

        // Record metrics
        metrics::counter!("confidence_updates_total").increment(1);
        if feedback_type.is_positive() {
            metrics::counter!("confidence_updates_positive").increment(1);
        } else {
            metrics::counter!("confidence_updates_negative").increment(1);
        }

        ConfidenceUpdate {
            element_id,
            old_confidence: current_confidence,
            new_confidence,
            delta,
        }
    }

    /// Apply multiple feedback signals to calculate cumulative update
    ///
    /// Useful when processing batch feedback or historical feedback for an element.
    #[instrument(skip(self, feedback_types))]
    pub fn calculate_cumulative_update(
        &self,
        element_id: Uuid,
        current_confidence: f32,
        feedback_types: impl IntoIterator<Item = FeedbackType>,
    ) -> ConfidenceUpdate {
        let mut total_delta: f32 = 0.0;

        for feedback_type in feedback_types {
            total_delta += feedback_type.confidence_delta();
        }

        let unclamped = current_confidence + total_delta;
        let new_confidence = self.clamp(unclamped);

        debug!(
            element_id = %element_id,
            current = current_confidence,
            total_delta = total_delta,
            new = new_confidence,
            "Calculated cumulative confidence update"
        );

        ConfidenceUpdate {
            element_id,
            old_confidence: current_confidence,
            new_confidence,
            delta: total_delta,
        }
    }

    /// Calculate a propagated confidence update (for related elements)
    ///
    /// Propagated updates apply a fraction of the original delta.
    /// Default propagation factor is 0.5 per hop.
    #[instrument(skip(self))]
    pub fn calculate_propagated_update(
        &self,
        element_id: Uuid,
        current_confidence: f32,
        original_delta: f32,
        propagation_factor: f32,
    ) -> ConfidenceUpdate {
        let propagated_delta = original_delta * propagation_factor;
        let unclamped = current_confidence + propagated_delta;
        let new_confidence = self.clamp(unclamped);

        debug!(
            element_id = %element_id,
            current = current_confidence,
            original_delta = original_delta,
            propagated_delta = propagated_delta,
            factor = propagation_factor,
            new = new_confidence,
            "Calculated propagated confidence update"
        );

        ConfidenceUpdate {
            element_id,
            old_confidence: current_confidence,
            new_confidence,
            delta: propagated_delta,
        }
    }

    /// Clamp a confidence value to the allowed range
    #[inline]
    fn clamp(&self, value: f32) -> f32 {
        value.clamp(self.min_confidence, self.max_confidence)
    }

    /// Get the minimum confidence bound
    pub fn min_confidence(&self) -> f32 {
        self.min_confidence
    }

    /// Get the maximum confidence bound
    pub fn max_confidence(&self) -> f32 {
        self.max_confidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbs_up_increases_confidence() {
        let updater = ConfidenceUpdater::new();
        let update = updater.calculate_update(
            Uuid::new_v4(),
            0.5,
            FeedbackType::ThumbsUp,
        );

        assert_eq!(update.old_confidence, 0.5);
        assert_eq!(update.new_confidence, 0.6);
        assert_eq!(update.delta, 0.1);
        assert!(update.is_increase());
    }

    #[test]
    fn test_thumbs_down_decreases_confidence() {
        let updater = ConfidenceUpdater::new();
        let update = updater.calculate_update(
            Uuid::new_v4(),
            0.5,
            FeedbackType::ThumbsDown,
        );

        assert_eq!(update.old_confidence, 0.5);
        assert_eq!(update.new_confidence, 0.35);
        assert_eq!(update.delta, -0.15);
        assert!(!update.is_increase());
    }

    #[test]
    fn test_clamp_at_max() {
        let updater = ConfidenceUpdater::new();
        let update = updater.calculate_update(
            Uuid::new_v4(),
            0.95,
            FeedbackType::ThumbsUp,
        );

        // Would be 1.05, clamped to 0.99
        assert_eq!(update.new_confidence, 0.99);
        assert!((update.actual_delta() - 0.04).abs() < 0.001); // 0.99 - 0.95
    }

    #[test]
    fn test_clamp_at_min() {
        let updater = ConfidenceUpdater::new();
        let update = updater.calculate_update(
            Uuid::new_v4(),
            0.15,
            FeedbackType::ThumbsDown,
        );

        // Would be 0.0, clamped to 0.1
        assert_eq!(update.new_confidence, 0.1);
        assert!((update.actual_delta() - (-0.05)).abs() < 0.001); // 0.1 - 0.15
    }

    #[test]
    fn test_cumulative_update() {
        let updater = ConfidenceUpdater::new();
        let feedback = vec![
            FeedbackType::ThumbsUp,
            FeedbackType::ThumbsUp,
            FeedbackType::ThumbsDown,
        ];

        let update = updater.calculate_cumulative_update(
            Uuid::new_v4(),
            0.5,
            feedback,
        );

        // 0.5 + 0.1 + 0.1 - 0.15 = 0.55
        assert_eq!(update.new_confidence, 0.55);
        assert!((update.delta - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn test_propagated_update() {
        let updater = ConfidenceUpdater::new();
        let update = updater.calculate_propagated_update(
            Uuid::new_v4(),
            0.5,
            0.1,  // original delta from thumbs up
            0.5,  // propagation factor
        );

        // 0.5 + (0.1 * 0.5) = 0.55
        assert_eq!(update.new_confidence, 0.55);
        assert_eq!(update.delta, 0.05);
    }

    #[test]
    fn test_custom_bounds() {
        let updater = ConfidenceUpdater::with_bounds(0.2, 0.8);

        // Test max bound
        let update = updater.calculate_update(
            Uuid::new_v4(),
            0.75,
            FeedbackType::ThumbsUp,
        );
        assert_eq!(update.new_confidence, 0.8);

        // Test min bound
        let update = updater.calculate_update(
            Uuid::new_v4(),
            0.25,
            FeedbackType::ThumbsDown,
        );
        assert_eq!(update.new_confidence, 0.2);
    }

    #[test]
    #[should_panic]
    fn test_invalid_bounds_panic() {
        ConfidenceUpdater::with_bounds(0.8, 0.2); // min > max
    }

    #[test]
    fn test_actual_delta_with_clamping() {
        let updater = ConfidenceUpdater::new();

        // When clamped, actual_delta differs from requested delta
        let update = updater.calculate_update(
            Uuid::new_v4(),
            0.98,
            FeedbackType::ThumbsUp,
        );

        assert_eq!(update.delta, 0.1); // Requested delta
        assert!((update.actual_delta() - 0.01).abs() < f32::EPSILON); // Actual change
    }
}
