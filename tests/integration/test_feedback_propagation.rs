//! Integration Test: Feedback → Confidence propagation
//!
//! This test validates that feedback affects element confidence scores.
//!
//! Prerequisites:
//! - Neo4j running (for element storage tests)
//!
//! Run with: cargo test --test test_feedback_propagation

use codegraph_feedback::{
    ConfidenceUpdate, ConfidenceUpdater, FeedbackType,
    DEFAULT_CONFIDENCE, MAX_CONFIDENCE, MIN_CONFIDENCE,
};
use uuid::Uuid;

#[tokio::test]
async fn test_positive_feedback_increases_confidence() {
    let updater = ConfidenceUpdater::new();
    let element_id = Uuid::new_v4();
    let initial_confidence = DEFAULT_CONFIDENCE; // 0.5

    // Submit positive feedback
    let update = updater.calculate_update(
        element_id,
        initial_confidence,
        FeedbackType::ThumbsUp,
    );

    // Verify confidence increased
    assert!(
        update.new_confidence > update.old_confidence,
        "Positive feedback should increase confidence"
    );
    assert_eq!(update.delta, 0.1, "ThumbsUp delta should be +0.1");
    assert_eq!(update.new_confidence, 0.6, "0.5 + 0.1 = 0.6");
    assert!(update.is_increase());
}

#[tokio::test]
async fn test_negative_feedback_decreases_confidence() {
    let updater = ConfidenceUpdater::new();
    let element_id = Uuid::new_v4();
    let initial_confidence = DEFAULT_CONFIDENCE; // 0.5

    // Submit negative feedback
    let update = updater.calculate_update(
        element_id,
        initial_confidence,
        FeedbackType::ThumbsDown,
    );

    // Verify confidence decreased
    assert!(
        update.new_confidence < update.old_confidence,
        "Negative feedback should decrease confidence"
    );
    assert_eq!(update.delta, -0.15, "ThumbsDown delta should be -0.15");
    assert_eq!(update.new_confidence, 0.35, "0.5 - 0.15 = 0.35");
    assert!(!update.is_increase());
}

#[tokio::test]
async fn test_confidence_clamped_at_maximum() {
    let updater = ConfidenceUpdater::new();
    let element_id = Uuid::new_v4();
    let high_confidence = 0.95;

    // Try to increase beyond maximum
    let update = updater.calculate_update(
        element_id,
        high_confidence,
        FeedbackType::ThumbsUp,
    );

    // Verify clamped at MAX_CONFIDENCE
    assert_eq!(
        update.new_confidence, MAX_CONFIDENCE,
        "Confidence should be clamped at maximum ({})",
        MAX_CONFIDENCE
    );
    assert!(update.new_confidence <= MAX_CONFIDENCE);
}

#[tokio::test]
async fn test_confidence_clamped_at_minimum() {
    let updater = ConfidenceUpdater::new();
    let element_id = Uuid::new_v4();
    let low_confidence = 0.15;

    // Try to decrease below minimum
    let update = updater.calculate_update(
        element_id,
        low_confidence,
        FeedbackType::ThumbsDown,
    );

    // Verify clamped at MIN_CONFIDENCE
    assert_eq!(
        update.new_confidence, MIN_CONFIDENCE,
        "Confidence should be clamped at minimum ({})",
        MIN_CONFIDENCE
    );
    assert!(update.new_confidence >= MIN_CONFIDENCE);
}

#[tokio::test]
async fn test_cumulative_feedback_effects() {
    let updater = ConfidenceUpdater::new();
    let element_id = Uuid::new_v4();
    let initial_confidence = 0.5;

    // Apply multiple feedback signals
    let feedback_sequence = vec![
        FeedbackType::ThumbsUp,   // +0.1
        FeedbackType::ThumbsUp,   // +0.1
        FeedbackType::ThumbsDown, // -0.15
    ];

    let update = updater.calculate_cumulative_update(
        element_id,
        initial_confidence,
        feedback_sequence,
    );

    // Net effect: 0.1 + 0.1 - 0.15 = 0.05
    assert!((update.delta - 0.05).abs() < f32::EPSILON);
    assert_eq!(update.new_confidence, 0.55);
}

#[tokio::test]
async fn test_propagated_confidence_update() {
    let updater = ConfidenceUpdater::new();
    let element_id = Uuid::new_v4();
    let initial_confidence = 0.5;
    let original_delta = 0.1; // From a thumbs up
    let propagation_factor = 0.5; // 50% propagation to related elements

    let update = updater.calculate_propagated_update(
        element_id,
        initial_confidence,
        original_delta,
        propagation_factor,
    );

    // Propagated delta: 0.1 * 0.5 = 0.05
    assert_eq!(update.delta, 0.05);
    assert_eq!(update.new_confidence, 0.55);
}

#[tokio::test]
async fn test_feedback_type_delta_values() {
    // Verify the delta values are as documented
    assert_eq!(
        FeedbackType::ThumbsUp.confidence_delta(),
        0.1,
        "ThumbsUp should have +0.1 delta"
    );
    assert_eq!(
        FeedbackType::ThumbsDown.confidence_delta(),
        -0.15,
        "ThumbsDown should have -0.15 delta"
    );
}

#[tokio::test]
async fn test_feedback_type_is_positive() {
    assert!(FeedbackType::ThumbsUp.is_positive());
    assert!(!FeedbackType::ThumbsDown.is_positive());
}

#[tokio::test]
async fn test_confidence_bounds() {
    // Verify the constants
    assert_eq!(MIN_CONFIDENCE, 0.1);
    assert_eq!(MAX_CONFIDENCE, 0.99);
    assert_eq!(DEFAULT_CONFIDENCE, 0.5);
}

#[tokio::test]
async fn test_custom_confidence_bounds() {
    let updater = ConfidenceUpdater::with_bounds(0.2, 0.8);

    // Test clamping at custom max
    let update = updater.calculate_update(
        Uuid::new_v4(),
        0.75,
        FeedbackType::ThumbsUp,
    );
    assert_eq!(update.new_confidence, 0.8);

    // Test clamping at custom min
    let update = updater.calculate_update(
        Uuid::new_v4(),
        0.25,
        FeedbackType::ThumbsDown,
    );
    assert_eq!(update.new_confidence, 0.2);
}

#[tokio::test]
async fn test_actual_delta_with_clamping() {
    let updater = ConfidenceUpdater::new();

    // When clamped, actual_delta differs from requested delta
    let update = updater.calculate_update(
        Uuid::new_v4(),
        0.98,
        FeedbackType::ThumbsUp,
    );

    // Requested delta is 0.1, but actual change is only 0.01 due to clamping
    assert_eq!(update.delta, 0.1);
    assert!((update.actual_delta() - 0.01).abs() < f32::EPSILON);
}

#[tokio::test]
async fn test_confidence_update_struct() {
    let update = ConfidenceUpdate {
        element_id: Uuid::new_v4(),
        old_confidence: 0.5,
        new_confidence: 0.6,
        delta: 0.1,
    };

    assert!(update.is_increase());
    // Use approximate comparison for floating point
    assert!((update.actual_delta() - 0.1).abs() < 0.001);
}

#[tokio::test]
async fn test_multiple_elements_independent_updates() {
    let updater = ConfidenceUpdater::new();

    // Create multiple elements with different initial confidences
    let elements = vec![
        (Uuid::new_v4(), 0.3),
        (Uuid::new_v4(), 0.5),
        (Uuid::new_v4(), 0.7),
    ];

    // Apply same feedback to all
    let updates: Vec<ConfidenceUpdate> = elements
        .iter()
        .map(|(id, confidence)| {
            updater.calculate_update(*id, *confidence, FeedbackType::ThumbsUp)
        })
        .collect();

    // Each should have increased by 0.1
    assert_eq!(updates[0].new_confidence, 0.4);
    assert_eq!(updates[1].new_confidence, 0.6);
    assert_eq!(updates[2].new_confidence, 0.8);
}

#[tokio::test]
async fn test_feedback_simulation_over_time() {
    let updater = ConfidenceUpdater::new();
    let element_id = Uuid::new_v4();
    let mut confidence = DEFAULT_CONFIDENCE;

    // Simulate a series of feedback over time
    let feedback_history = vec![
        FeedbackType::ThumbsUp,   // 0.5 -> 0.6
        FeedbackType::ThumbsUp,   // 0.6 -> 0.7
        FeedbackType::ThumbsDown, // 0.7 -> 0.55
        FeedbackType::ThumbsUp,   // 0.55 -> 0.65
        FeedbackType::ThumbsUp,   // 0.65 -> 0.75
    ];

    for feedback in feedback_history {
        let update = updater.calculate_update(element_id, confidence, feedback);
        confidence = update.new_confidence;
    }

    // Final confidence after all feedback
    assert!((confidence - 0.75).abs() < 0.01);
}

#[tokio::test]
async fn test_nars_truth_value_consistency() {
    // NARS truth values are <frequency, confidence>
    // Frequency represents how often the statement is true
    // Confidence represents how much evidence we have

    let updater = ConfidenceUpdater::new();
    let element_id = Uuid::new_v4();

    // Simulate NARS-style reasoning
    // Initial: frequency=0.9, confidence=0.5 (partially trusted high-quality element)
    let initial_confidence = 0.5;

    // Positive feedback increases confidence (more evidence)
    let update = updater.calculate_update(
        element_id,
        initial_confidence,
        FeedbackType::ThumbsUp,
    );

    // The confidence should increase, indicating more evidence
    assert!(update.new_confidence > initial_confidence);

    // The system should maintain bounded confidence (can't be 100% certain)
    assert!(update.new_confidence < 1.0);
}
