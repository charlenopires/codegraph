//! RewardComputer - Calculates composite reward scores for elements
//!
//! The reward score is used to rank elements for retrieval and generation.
//! It combines multiple signals:
//!
//! - **Base Confidence (40%)**: The element's current NARS confidence value
//! - **Similarity Bonus (30%)**: Bonus for being similar to high-confidence elements
//! - **Connectivity Bonus (20%)**: Bonus for being well-connected in the graph
//! - **Negative Penalty (10%)**: Deduction for negative feedback history

use tracing::{debug, instrument};
use uuid::Uuid;

/// Weight for base confidence in reward calculation
pub const WEIGHT_BASE_CONFIDENCE: f32 = 0.40;

/// Weight for similarity bonus in reward calculation
pub const WEIGHT_SIMILARITY_BONUS: f32 = 0.30;

/// Weight for connectivity bonus in reward calculation
pub const WEIGHT_CONNECTIVITY_BONUS: f32 = 0.20;

/// Weight for negative penalty in reward calculation
pub const WEIGHT_NEGATIVE_PENALTY: f32 = 0.10;

/// Input signals for reward computation
#[derive(Debug, Clone, Default)]
pub struct RewardSignals {
    /// Element's current NARS confidence (0.0 - 1.0)
    pub base_confidence: f32,

    /// Average confidence of SIMILAR_TO neighbors (0.0 - 1.0)
    pub avg_similar_confidence: f32,

    /// Number of connections (normalized to 0.0 - 1.0)
    pub connectivity_score: f32,

    /// Ratio of negative feedback (0.0 - 1.0)
    pub negative_feedback_ratio: f32,
}

impl RewardSignals {
    /// Create signals with just base confidence
    pub fn from_confidence(confidence: f32) -> Self {
        Self {
            base_confidence: confidence,
            ..Default::default()
        }
    }

    /// Builder: set similarity score
    pub fn with_similarity(mut self, avg_similar_confidence: f32) -> Self {
        self.avg_similar_confidence = avg_similar_confidence;
        self
    }

    /// Builder: set connectivity score
    pub fn with_connectivity(mut self, connectivity_score: f32) -> Self {
        self.connectivity_score = connectivity_score;
        self
    }

    /// Builder: set negative feedback ratio
    pub fn with_negative_ratio(mut self, negative_feedback_ratio: f32) -> Self {
        self.negative_feedback_ratio = negative_feedback_ratio;
        self
    }
}

/// Computed reward with component breakdown
#[derive(Debug, Clone)]
pub struct RewardResult {
    /// Element ID
    pub element_id: Uuid,

    /// Final composite reward score (0.0 - 1.0)
    pub reward: f32,

    /// Base confidence component
    pub base_component: f32,

    /// Similarity bonus component
    pub similarity_component: f32,

    /// Connectivity bonus component
    pub connectivity_component: f32,

    /// Negative penalty component (subtracted)
    pub penalty_component: f32,
}

impl RewardResult {
    /// Returns true if the element has a high reward (> 0.7)
    pub fn is_high_reward(&self) -> bool {
        self.reward > 0.7
    }

    /// Returns true if the element has a low reward (< 0.3)
    pub fn is_low_reward(&self) -> bool {
        self.reward < 0.3
    }

    /// Returns the dominant factor (largest absolute contribution)
    pub fn dominant_factor(&self) -> &'static str {
        let components = [
            (self.base_component, "base_confidence"),
            (self.similarity_component, "similarity_bonus"),
            (self.connectivity_component, "connectivity_bonus"),
            (self.penalty_component, "negative_penalty"),
        ];

        components
            .into_iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, name)| name)
            .unwrap_or("base_confidence")
    }
}

/// Configuration for reward weights
#[derive(Debug, Clone)]
pub struct RewardWeights {
    /// Weight for base confidence (default: 0.40)
    pub base_confidence: f32,
    /// Weight for similarity bonus (default: 0.30)
    pub similarity_bonus: f32,
    /// Weight for connectivity bonus (default: 0.20)
    pub connectivity_bonus: f32,
    /// Weight for negative penalty (default: 0.10)
    pub negative_penalty: f32,
}

impl Default for RewardWeights {
    fn default() -> Self {
        Self {
            base_confidence: WEIGHT_BASE_CONFIDENCE,
            similarity_bonus: WEIGHT_SIMILARITY_BONUS,
            connectivity_bonus: WEIGHT_CONNECTIVITY_BONUS,
            negative_penalty: WEIGHT_NEGATIVE_PENALTY,
        }
    }
}

impl RewardWeights {
    /// Validate that weights sum to 1.0
    pub fn is_valid(&self) -> bool {
        let sum = self.base_confidence + self.similarity_bonus + self.connectivity_bonus + self.negative_penalty;
        (sum - 1.0).abs() < 0.001
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.base_confidence + self.similarity_bonus + self.connectivity_bonus + self.negative_penalty;
        if sum > 0.0 {
            self.base_confidence /= sum;
            self.similarity_bonus /= sum;
            self.connectivity_bonus /= sum;
            self.negative_penalty /= sum;
        }
    }
}

/// Computes composite reward scores for elements
#[derive(Debug, Clone)]
pub struct RewardComputer {
    weights: RewardWeights,
}

impl Default for RewardComputer {
    fn default() -> Self {
        Self::new()
    }
}

impl RewardComputer {
    /// Create a new RewardComputer with default weights
    ///
    /// Default weights:
    /// - Base confidence: 40%
    /// - Similarity bonus: 30%
    /// - Connectivity bonus: 20%
    /// - Negative penalty: 10%
    pub fn new() -> Self {
        Self {
            weights: RewardWeights::default(),
        }
    }

    /// Create a RewardComputer with custom weights
    pub fn with_weights(weights: RewardWeights) -> Self {
        assert!(weights.is_valid(), "Weights must sum to 1.0");
        Self { weights }
    }

    /// Compute the reward score for an element
    ///
    /// Formula:
    /// ```text
    /// reward = base_confidence * 0.4
    ///        + similarity_bonus * 0.3
    ///        + connectivity_bonus * 0.2
    ///        - negative_penalty * 0.1
    /// ```
    #[instrument(skip(self))]
    pub fn compute(&self, element_id: Uuid, signals: &RewardSignals) -> RewardResult {
        // Calculate each component
        let base_component = signals.base_confidence * self.weights.base_confidence;
        let similarity_component = signals.avg_similar_confidence * self.weights.similarity_bonus;
        let connectivity_component = signals.connectivity_score * self.weights.connectivity_bonus;
        let penalty_component = signals.negative_feedback_ratio * self.weights.negative_penalty;

        // Composite reward (penalty is subtracted)
        let reward = (base_component + similarity_component + connectivity_component - penalty_component)
            .clamp(0.0, 1.0);

        debug!(
            element_id = %element_id,
            reward = reward,
            base = base_component,
            similarity = similarity_component,
            connectivity = connectivity_component,
            penalty = penalty_component,
            "Computed reward"
        );

        // Record metrics
        metrics::histogram!("reward_scores").record(reward as f64);
        metrics::histogram!("reward_base_component").record(base_component as f64);
        metrics::histogram!("reward_similarity_component").record(similarity_component as f64);
        metrics::histogram!("reward_connectivity_component").record(connectivity_component as f64);
        metrics::histogram!("reward_penalty_component").record(penalty_component as f64);

        RewardResult {
            element_id,
            reward,
            base_component,
            similarity_component,
            connectivity_component,
            penalty_component,
        }
    }

    /// Compute rewards for multiple elements and sort by reward descending
    pub fn compute_batch(&self, elements: Vec<(Uuid, RewardSignals)>) -> Vec<RewardResult> {
        let mut results: Vec<_> = elements
            .into_iter()
            .map(|(id, signals)| self.compute(id, &signals))
            .collect();

        results.sort_by(|a, b| b.reward.partial_cmp(&a.reward).unwrap());
        results
    }

    /// Get the current weights
    pub fn weights(&self) -> &RewardWeights {
        &self.weights
    }

    /// Compute a quick reward estimate using only base confidence
    ///
    /// Useful for fast filtering before full computation.
    pub fn quick_estimate(&self, base_confidence: f32) -> f32 {
        base_confidence * self.weights.base_confidence
    }
}

/// Normalize a connection count to a 0-1 score
///
/// Uses logarithmic scaling: score = log(1 + count) / log(1 + max_connections)
pub fn normalize_connectivity(connection_count: u32, max_connections: u32) -> f32 {
    if max_connections == 0 {
        return 0.0;
    }

    let log_count = (1.0 + connection_count as f64).ln();
    let log_max = (1.0 + max_connections as f64).ln();

    (log_count / log_max) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_weights() {
        let weights = RewardWeights::default();
        assert!(weights.is_valid());
        assert_eq!(weights.base_confidence, 0.40);
        assert_eq!(weights.similarity_bonus, 0.30);
        assert_eq!(weights.connectivity_bonus, 0.20);
        assert_eq!(weights.negative_penalty, 0.10);
    }

    #[test]
    fn test_basic_reward_computation() {
        let computer = RewardComputer::new();
        let signals = RewardSignals {
            base_confidence: 0.8,
            avg_similar_confidence: 0.7,
            connectivity_score: 0.5,
            negative_feedback_ratio: 0.1,
        };

        let result = computer.compute(Uuid::new_v4(), &signals);

        // 0.8 * 0.4 + 0.7 * 0.3 + 0.5 * 0.2 - 0.1 * 0.1
        // = 0.32 + 0.21 + 0.10 - 0.01 = 0.62
        assert!((result.reward - 0.62).abs() < 0.001);
        assert!((result.base_component - 0.32).abs() < 0.001);
        assert!((result.similarity_component - 0.21).abs() < 0.001);
        assert!((result.connectivity_component - 0.10).abs() < 0.001);
        assert!((result.penalty_component - 0.01).abs() < 0.001);
    }

    #[test]
    fn test_high_confidence_element() {
        let computer = RewardComputer::new();
        let signals = RewardSignals {
            base_confidence: 1.0,
            avg_similar_confidence: 1.0,
            connectivity_score: 1.0,
            negative_feedback_ratio: 0.0,
        };

        let result = computer.compute(Uuid::new_v4(), &signals);

        // 1.0 * 0.4 + 1.0 * 0.3 + 1.0 * 0.2 - 0.0 * 0.1 = 0.9
        assert!((result.reward - 0.9).abs() < 0.001);
        assert!(result.is_high_reward());
    }

    #[test]
    fn test_low_confidence_element() {
        let computer = RewardComputer::new();
        let signals = RewardSignals {
            base_confidence: 0.2,
            avg_similar_confidence: 0.1,
            connectivity_score: 0.1,
            negative_feedback_ratio: 0.8,
        };

        let result = computer.compute(Uuid::new_v4(), &signals);

        // 0.2 * 0.4 + 0.1 * 0.3 + 0.1 * 0.2 - 0.8 * 0.1
        // = 0.08 + 0.03 + 0.02 - 0.08 = 0.05
        assert!((result.reward - 0.05).abs() < 0.001);
        assert!(result.is_low_reward());
    }

    #[test]
    fn test_high_penalty_clamped() {
        let computer = RewardComputer::new();
        let signals = RewardSignals {
            base_confidence: 0.0,
            avg_similar_confidence: 0.0,
            connectivity_score: 0.0,
            negative_feedback_ratio: 1.0,
        };

        let result = computer.compute(Uuid::new_v4(), &signals);

        // Would be -0.1, clamped to 0.0
        assert_eq!(result.reward, 0.0);
    }

    #[test]
    fn test_batch_computation() {
        let computer = RewardComputer::new();
        let elements = vec![
            (Uuid::new_v4(), RewardSignals::from_confidence(0.3)),
            (Uuid::new_v4(), RewardSignals::from_confidence(0.9)),
            (Uuid::new_v4(), RewardSignals::from_confidence(0.5)),
        ];

        let results = computer.compute_batch(elements);

        // Should be sorted by reward descending
        assert!(results[0].reward >= results[1].reward);
        assert!(results[1].reward >= results[2].reward);
    }

    #[test]
    fn test_signals_builder() {
        let signals = RewardSignals::from_confidence(0.8)
            .with_similarity(0.7)
            .with_connectivity(0.5)
            .with_negative_ratio(0.1);

        assert_eq!(signals.base_confidence, 0.8);
        assert_eq!(signals.avg_similar_confidence, 0.7);
        assert_eq!(signals.connectivity_score, 0.5);
        assert_eq!(signals.negative_feedback_ratio, 0.1);
    }

    #[test]
    fn test_normalize_connectivity() {
        // Zero connections
        assert_eq!(normalize_connectivity(0, 100), 0.0);

        // Max connections
        let score = normalize_connectivity(100, 100);
        assert!((score - 1.0).abs() < 0.001);

        // Mid-range (logarithmic)
        let score = normalize_connectivity(10, 100);
        assert!(score > 0.0 && score < 1.0);

        // Edge case: max_connections = 0
        assert_eq!(normalize_connectivity(10, 0), 0.0);
    }

    #[test]
    fn test_dominant_factor() {
        let computer = RewardComputer::new();

        // High base confidence dominates
        let signals = RewardSignals {
            base_confidence: 1.0,
            avg_similar_confidence: 0.0,
            connectivity_score: 0.0,
            negative_feedback_ratio: 0.0,
        };
        let result = computer.compute(Uuid::new_v4(), &signals);
        assert_eq!(result.dominant_factor(), "base_confidence");

        // High similarity dominates
        let signals = RewardSignals {
            base_confidence: 0.0,
            avg_similar_confidence: 1.0,
            connectivity_score: 0.0,
            negative_feedback_ratio: 0.0,
        };
        let result = computer.compute(Uuid::new_v4(), &signals);
        assert_eq!(result.dominant_factor(), "similarity_bonus");
    }

    #[test]
    fn test_quick_estimate() {
        let computer = RewardComputer::new();

        assert!((computer.quick_estimate(1.0) - 0.4).abs() < 0.001);
        assert!((computer.quick_estimate(0.5) - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_weight_normalization() {
        let mut weights = RewardWeights {
            base_confidence: 0.4,
            similarity_bonus: 0.3,
            connectivity_bonus: 0.2,
            negative_penalty: 0.5, // Sum = 1.4
        };

        assert!(!weights.is_valid());

        weights.normalize();

        assert!(weights.is_valid());
        assert!((weights.base_confidence - 0.286).abs() < 0.01);
    }

    #[test]
    #[should_panic]
    fn test_invalid_weights_panic() {
        let weights = RewardWeights {
            base_confidence: 0.5,
            similarity_bonus: 0.5,
            connectivity_bonus: 0.5,
            negative_penalty: 0.5,
        };
        RewardComputer::with_weights(weights);
    }
}
