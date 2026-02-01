//! Ranker - scores and ranks retrieval results
//!
//! Formula: 0.5*narsese_confidence + 0.3*semantic_similarity + 0.2*graph_degree

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A retrieved element with scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredElement {
    /// Unique element ID
    pub element_id: String,
    /// Element name
    pub name: String,
    /// Component category
    pub category: String,
    /// Tags/attributes
    pub tags: Vec<String>,
    /// NARS reasoning confidence (0.0-1.0)
    pub narsese_confidence: f32,
    /// Vector similarity score (0.0-1.0)
    pub semantic_similarity: f32,
    /// Graph connectivity degree (normalized 0.0-1.0)
    pub graph_degree: f32,
    /// Combined final score
    pub final_score: f32,
    /// Source of the result (vector, graph, fulltext)
    pub source: ResultSource,
}

/// Source of the retrieval result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResultSource {
    Vector,
    Graph,
    Fulltext,
    Hybrid,
}

/// Weights for scoring components
#[derive(Debug, Clone, Copy)]
pub struct RankingWeights {
    pub narsese_confidence: f32,
    pub semantic_similarity: f32,
    pub graph_degree: f32,
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            narsese_confidence: 0.5,
            semantic_similarity: 0.3,
            graph_degree: 0.2,
        }
    }
}

/// Ranker for combining and scoring retrieval results
pub struct Ranker {
    weights: RankingWeights,
}

impl Default for Ranker {
    fn default() -> Self {
        Self::new()
    }
}

impl Ranker {
    pub fn new() -> Self {
        Self {
            weights: RankingWeights::default(),
        }
    }

    pub fn with_weights(weights: RankingWeights) -> Self {
        Self { weights }
    }

    /// Calculate final score for an element
    pub fn calculate_score(
        &self,
        narsese_confidence: f32,
        semantic_similarity: f32,
        graph_degree: f32,
    ) -> f32 {
        self.weights.narsese_confidence * narsese_confidence
            + self.weights.semantic_similarity * semantic_similarity
            + self.weights.graph_degree * graph_degree
    }

    /// Rank a list of elements by their scores
    pub fn rank(&self, mut elements: Vec<ScoredElement>) -> Vec<ScoredElement> {
        // Calculate final scores
        for elem in &mut elements {
            elem.final_score = self.calculate_score(
                elem.narsese_confidence,
                elem.semantic_similarity,
                elem.graph_degree,
            );
        }

        // Sort by final score descending
        elements.sort_by(|a, b| {
            b.final_score
                .partial_cmp(&a.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        elements
    }

    /// Deduplicate elements by element_id, keeping highest score
    pub fn deduplicate(&self, elements: Vec<ScoredElement>) -> Vec<ScoredElement> {
        let mut best_by_id: HashMap<String, ScoredElement> = HashMap::new();

        for elem in elements {
            best_by_id
                .entry(elem.element_id.clone())
                .and_modify(|existing| {
                    if elem.final_score > existing.final_score {
                        *existing = elem.clone();
                    }
                })
                .or_insert(elem);
        }

        let mut results: Vec<_> = best_by_id.into_values().collect();
        results.sort_by(|a, b| {
            b.final_score
                .partial_cmp(&a.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Rank and deduplicate in one step
    pub fn rank_and_deduplicate(&self, elements: Vec<ScoredElement>) -> Vec<ScoredElement> {
        let ranked = self.rank(elements);
        self.deduplicate(ranked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_score() {
        let ranker = Ranker::new();
        let score = ranker.calculate_score(0.8, 0.6, 0.4);
        // 0.5*0.8 + 0.3*0.6 + 0.2*0.4 = 0.4 + 0.18 + 0.08 = 0.66
        assert!((score - 0.66).abs() < 0.001);
    }

    #[test]
    fn test_deduplicate() {
        let ranker = Ranker::new();
        let elements = vec![
            ScoredElement {
                element_id: "1".to_string(),
                name: "Button".to_string(),
                category: "button".to_string(),
                tags: vec![],
                narsese_confidence: 0.8,
                semantic_similarity: 0.6,
                graph_degree: 0.4,
                final_score: 0.5,
                source: ResultSource::Vector,
            },
            ScoredElement {
                element_id: "1".to_string(),
                name: "Button".to_string(),
                category: "button".to_string(),
                tags: vec![],
                narsese_confidence: 0.9,
                semantic_similarity: 0.7,
                graph_degree: 0.5,
                final_score: 0.7,
                source: ResultSource::Graph,
            },
        ];

        let result = ranker.deduplicate(elements);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].final_score, 0.7);
    }
}
