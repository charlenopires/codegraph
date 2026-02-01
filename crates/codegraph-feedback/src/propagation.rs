//! ConfidencePropagator - Propagates confidence updates through the knowledge graph
//!
//! When an element receives feedback, related elements also receive a portion
//! of the confidence update, decaying with distance:
//!
//! - Hop 1: delta * 0.5 (50% of original)
//! - Hop 2: delta * 0.25 (25% of original)
//!
//! Propagation follows SIMILAR_TO and CAN_REPLACE relationships.

use std::collections::{HashMap, HashSet};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::confidence::{ConfidenceUpdate, ConfidenceUpdater};

/// Default propagation decay factor per hop
pub const DEFAULT_DECAY_FACTOR: f32 = 0.5;

/// Default maximum propagation depth (hops)
pub const DEFAULT_MAX_HOPS: u32 = 2;

/// Relationship types that propagate confidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropagationRelation {
    /// Elements are similar to each other
    SimilarTo,
    /// One element can replace another
    CanReplace,
}

impl std::fmt::Display for PropagationRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropagationRelation::SimilarTo => write!(f, "SIMILAR_TO"),
            PropagationRelation::CanReplace => write!(f, "CAN_REPLACE"),
        }
    }
}

/// A related element with its relationship type
#[derive(Debug, Clone)]
pub struct RelatedElement {
    /// Element ID
    pub element_id: Uuid,
    /// Type of relationship
    pub relation: PropagationRelation,
    /// Current confidence of the element
    pub current_confidence: f32,
}

/// Result of propagating confidence to related elements
#[derive(Debug, Clone)]
pub struct PropagationResult {
    /// The source element that received the original feedback
    pub source_element_id: Uuid,
    /// The original delta applied to the source
    pub original_delta: f32,
    /// Updates to be applied to related elements, keyed by element ID
    pub propagated_updates: HashMap<Uuid, ConfidenceUpdate>,
    /// Total number of elements affected (including source)
    pub total_affected: usize,
    /// Maximum hop depth reached
    pub max_depth_reached: u32,
}

impl PropagationResult {
    /// Returns the total confidence delta applied across all elements
    pub fn total_delta_applied(&self) -> f32 {
        self.propagated_updates
            .values()
            .map(|u| u.actual_delta())
            .sum::<f32>()
            + self.original_delta
    }

    /// Returns updates grouped by hop distance
    pub fn updates_by_hop(&self) -> HashMap<u32, Vec<&ConfidenceUpdate>> {
        // Note: This would require tracking hop distance per update
        // For now, returns all updates in a single group
        let mut result = HashMap::new();
        let updates: Vec<_> = self.propagated_updates.values().collect();
        if !updates.is_empty() {
            result.insert(1, updates);
        }
        result
    }
}

/// Trait for resolving related elements from the graph
///
/// Implement this trait to connect the propagator to your graph storage.
#[async_trait::async_trait]
pub trait RelationResolver: Send + Sync {
    /// Get elements related to the given element via propagation relationships
    ///
    /// Should return elements connected by SIMILAR_TO or CAN_REPLACE relationships.
    async fn get_related_elements(&self, element_id: Uuid) -> Vec<RelatedElement>;

    /// Get current confidence for an element
    async fn get_confidence(&self, element_id: Uuid) -> Option<f32>;

    /// Update confidence for multiple elements
    async fn update_confidences(&self, updates: &[ConfidenceUpdate]) -> Result<(), String>;
}

/// Propagates confidence updates through the knowledge graph
#[derive(Debug, Clone)]
pub struct ConfidencePropagator {
    /// Decay factor per hop (default: 0.5)
    decay_factor: f32,
    /// Maximum propagation depth in hops (default: 2)
    max_hops: u32,
    /// The confidence updater for calculating individual updates
    updater: ConfidenceUpdater,
}

impl Default for ConfidencePropagator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfidencePropagator {
    /// Create a new ConfidencePropagator with default settings
    ///
    /// - Decay factor: 0.5 (50% per hop)
    /// - Max hops: 2
    pub fn new() -> Self {
        Self {
            decay_factor: DEFAULT_DECAY_FACTOR,
            max_hops: DEFAULT_MAX_HOPS,
            updater: ConfidenceUpdater::new(),
        }
    }

    /// Create a ConfidencePropagator with custom settings
    pub fn with_config(decay_factor: f32, max_hops: u32) -> Self {
        assert!(
            (0.0..=1.0).contains(&decay_factor),
            "Decay factor must be between 0 and 1"
        );
        Self {
            decay_factor,
            max_hops,
            updater: ConfidenceUpdater::new(),
        }
    }

    /// Calculate propagation effects for a confidence delta
    ///
    /// This method calculates what updates would be applied to related elements
    /// without actually applying them. Use this to preview changes or for batch processing.
    #[instrument(skip(self, related_elements))]
    pub fn calculate_propagation(
        &self,
        source_element_id: Uuid,
        original_delta: f32,
        related_elements: &[(u32, RelatedElement)], // (hop_distance, element)
    ) -> PropagationResult {
        let mut propagated_updates = HashMap::new();
        let mut max_depth_reached = 0u32;
        let mut visited = HashSet::new();
        visited.insert(source_element_id);

        for (hop_distance, related) in related_elements {
            // Skip if already visited (prevent cycles)
            if visited.contains(&related.element_id) {
                continue;
            }
            visited.insert(related.element_id);

            // Skip if beyond max hops
            if *hop_distance > self.max_hops {
                continue;
            }

            // Calculate propagated delta: original_delta * decay_factor^hop_distance
            let propagation_factor = self.decay_factor.powi(*hop_distance as i32);
            let update = self.updater.calculate_propagated_update(
                related.element_id,
                related.current_confidence,
                original_delta,
                propagation_factor,
            );

            debug!(
                element_id = %related.element_id,
                hop = hop_distance,
                relation = %related.relation,
                original_delta = original_delta,
                propagated_delta = update.delta,
                new_confidence = update.new_confidence,
                "Calculated propagation"
            );

            propagated_updates.insert(related.element_id, update);
            max_depth_reached = max_depth_reached.max(*hop_distance);
        }

        let total_affected = propagated_updates.len() + 1; // +1 for source

        // Record metrics
        metrics::counter!("confidence_propagations_total").increment(1);
        metrics::histogram!("confidence_propagation_affected_count").record(total_affected as f64);
        metrics::histogram!("confidence_propagation_max_depth").record(max_depth_reached as f64);

        info!(
            source = %source_element_id,
            original_delta = original_delta,
            affected_count = total_affected,
            max_depth = max_depth_reached,
            "Confidence propagation calculated"
        );

        PropagationResult {
            source_element_id,
            original_delta,
            propagated_updates,
            total_affected,
            max_depth_reached,
        }
    }

    /// Propagate confidence through the graph using a resolver
    ///
    /// This method:
    /// 1. Fetches related elements from the graph
    /// 2. Calculates propagated updates
    /// 3. Applies updates to the graph
    #[instrument(skip(self, resolver))]
    pub async fn propagate<R: RelationResolver>(
        &self,
        source_element_id: Uuid,
        original_delta: f32,
        resolver: &R,
    ) -> Result<PropagationResult, String> {
        // Collect related elements up to max_hops using BFS
        let mut visited = HashSet::new();
        let mut to_visit = vec![(source_element_id, 0u32)];
        let mut related_with_distance = Vec::new();

        visited.insert(source_element_id);

        while let Some((current_id, current_hop)) = to_visit.pop() {
            if current_hop >= self.max_hops {
                continue;
            }

            let next_hop = current_hop + 1;
            let related = resolver.get_related_elements(current_id).await;

            for elem in related {
                if !visited.contains(&elem.element_id) {
                    visited.insert(elem.element_id);
                    related_with_distance.push((next_hop, elem.clone()));

                    // Queue for further exploration if within max hops
                    if next_hop < self.max_hops {
                        to_visit.push((elem.element_id, next_hop));
                    }
                }
            }
        }

        // Calculate propagation
        let result = self.calculate_propagation(
            source_element_id,
            original_delta,
            &related_with_distance,
        );

        // Apply updates to the graph
        let updates: Vec<_> = result.propagated_updates.values().cloned().collect();
        if !updates.is_empty() {
            resolver.update_confidences(&updates).await?;
        }

        Ok(result)
    }

    /// Get the decay factor
    pub fn decay_factor(&self) -> f32 {
        self.decay_factor
    }

    /// Get the maximum hops
    pub fn max_hops(&self) -> u32 {
        self.max_hops
    }

    /// Calculate the propagation factor for a given hop distance
    pub fn propagation_factor_at_hop(&self, hop: u32) -> f32 {
        self.decay_factor.powi(hop as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_related(id: Uuid, relation: PropagationRelation, confidence: f32) -> RelatedElement {
        RelatedElement {
            element_id: id,
            relation,
            current_confidence: confidence,
        }
    }

    #[test]
    fn test_propagation_factor_decay() {
        let propagator = ConfidencePropagator::new();

        // Default decay is 0.5
        assert_eq!(propagator.propagation_factor_at_hop(0), 1.0);
        assert_eq!(propagator.propagation_factor_at_hop(1), 0.5);
        assert_eq!(propagator.propagation_factor_at_hop(2), 0.25);
        assert_eq!(propagator.propagation_factor_at_hop(3), 0.125);
    }

    #[test]
    fn test_single_hop_propagation() {
        let propagator = ConfidencePropagator::new();
        let source_id = Uuid::new_v4();
        let related_id = Uuid::new_v4();

        let related = vec![(
            1,
            make_related(related_id, PropagationRelation::SimilarTo, 0.5),
        )];

        let result = propagator.calculate_propagation(source_id, 0.1, &related);

        assert_eq!(result.source_element_id, source_id);
        assert_eq!(result.original_delta, 0.1);
        assert_eq!(result.total_affected, 2); // source + 1 related
        assert_eq!(result.max_depth_reached, 1);

        let update = result.propagated_updates.get(&related_id).unwrap();
        assert_eq!(update.delta, 0.05); // 0.1 * 0.5
        assert_eq!(update.new_confidence, 0.55); // 0.5 + 0.05
    }

    #[test]
    fn test_two_hop_propagation() {
        let propagator = ConfidencePropagator::new();
        let source_id = Uuid::new_v4();
        let hop1_id = Uuid::new_v4();
        let hop2_id = Uuid::new_v4();

        let related = vec![
            (1, make_related(hop1_id, PropagationRelation::SimilarTo, 0.5)),
            (2, make_related(hop2_id, PropagationRelation::CanReplace, 0.5)),
        ];

        let result = propagator.calculate_propagation(source_id, 0.1, &related);

        assert_eq!(result.total_affected, 3);
        assert_eq!(result.max_depth_reached, 2);

        // Hop 1: 0.1 * 0.5 = 0.05
        let update1 = result.propagated_updates.get(&hop1_id).unwrap();
        assert_eq!(update1.delta, 0.05);

        // Hop 2: 0.1 * 0.25 = 0.025
        let update2 = result.propagated_updates.get(&hop2_id).unwrap();
        assert_eq!(update2.delta, 0.025);
    }

    #[test]
    fn test_max_hops_limit() {
        let propagator = ConfidencePropagator::with_config(0.5, 2);
        let source_id = Uuid::new_v4();
        let hop3_id = Uuid::new_v4();

        // Element at hop 3 should be ignored
        let related = vec![(
            3,
            make_related(hop3_id, PropagationRelation::SimilarTo, 0.5),
        )];

        let result = propagator.calculate_propagation(source_id, 0.1, &related);

        assert_eq!(result.total_affected, 1); // Only source
        assert!(result.propagated_updates.is_empty());
    }

    #[test]
    fn test_cycle_prevention() {
        let propagator = ConfidencePropagator::new();
        let source_id = Uuid::new_v4();

        // Source appears in related (cycle)
        let related = vec![(
            1,
            make_related(source_id, PropagationRelation::SimilarTo, 0.5),
        )];

        let result = propagator.calculate_propagation(source_id, 0.1, &related);

        // Source should not be updated again
        assert!(result.propagated_updates.is_empty());
    }

    #[test]
    fn test_negative_delta_propagation() {
        let propagator = ConfidencePropagator::new();
        let source_id = Uuid::new_v4();
        let related_id = Uuid::new_v4();

        let related = vec![(
            1,
            make_related(related_id, PropagationRelation::SimilarTo, 0.5),
        )];

        // Negative delta (thumbs down)
        let result = propagator.calculate_propagation(source_id, -0.15, &related);

        let update = result.propagated_updates.get(&related_id).unwrap();
        assert_eq!(update.delta, -0.075); // -0.15 * 0.5
        assert_eq!(update.new_confidence, 0.425); // 0.5 - 0.075
    }

    #[test]
    fn test_custom_decay_factor() {
        let propagator = ConfidencePropagator::with_config(0.7, 2);
        let source_id = Uuid::new_v4();
        let related_id = Uuid::new_v4();

        let related = vec![(
            1,
            make_related(related_id, PropagationRelation::SimilarTo, 0.5),
        )];

        let result = propagator.calculate_propagation(source_id, 0.1, &related);

        let update = result.propagated_updates.get(&related_id).unwrap();
        assert!((update.delta - 0.07).abs() < 0.001); // 0.1 * 0.7
    }

    #[test]
    fn test_total_delta_applied() {
        let propagator = ConfidencePropagator::new();
        let source_id = Uuid::new_v4();

        let related = vec![
            (1, make_related(Uuid::new_v4(), PropagationRelation::SimilarTo, 0.5)),
            (1, make_related(Uuid::new_v4(), PropagationRelation::SimilarTo, 0.5)),
        ];

        let result = propagator.calculate_propagation(source_id, 0.1, &related);

        // Original: 0.1 + 2 * 0.05 = 0.2
        assert!((result.total_delta_applied() - 0.2).abs() < 0.001);
    }

    #[test]
    #[should_panic]
    fn test_invalid_decay_factor() {
        ConfidencePropagator::with_config(1.5, 2); // > 1.0
    }
}
