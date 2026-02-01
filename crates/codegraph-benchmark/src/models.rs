//! Benchmark data models

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A benchmark query with expected results (ground truth)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkQuery {
    /// Unique query ID
    pub id: Uuid,
    /// Query text (natural language description)
    pub query: String,
    /// Design system filter (optional)
    pub design_system: Option<String>,
    /// Category filter (optional)
    pub category: Option<String>,
    /// Expected element IDs (ground truth)
    pub expected_ids: Vec<Uuid>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

impl BenchmarkQuery {
    /// Create a new benchmark query
    pub fn new(query: impl Into<String>, expected_ids: Vec<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            query: query.into(),
            design_system: None,
            category: None,
            expected_ids,
            tags: Vec::new(),
        }
    }

    /// Set design system filter
    pub fn with_design_system(mut self, design_system: impl Into<String>) -> Self {
        self.design_system = Some(design_system.into());
        self
    }

    /// Set category filter
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Result from executing a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Query ID
    pub query_id: Uuid,
    /// Returned element IDs
    pub returned_ids: Vec<Uuid>,
    /// Execution time in milliseconds
    pub latency_ms: f64,
    /// Number of results requested
    pub limit: u64,
}

impl QueryResult {
    /// Create a new query result
    pub fn new(query_id: Uuid, returned_ids: Vec<Uuid>, latency_ms: f64, limit: u64) -> Self {
        Self {
            query_id,
            returned_ids,
            latency_ms,
            limit,
        }
    }
}

/// Metrics for a single query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    /// Query ID
    pub query_id: Uuid,
    /// Precision (relevant returned / total returned)
    pub precision: f64,
    /// Recall (relevant returned / total relevant)
    pub recall: f64,
    /// F1 score (harmonic mean of precision and recall)
    pub f1_score: f64,
    /// Hallucination rate (irrelevant returned / total returned)
    pub hallucination_rate: f64,
    /// Latency in milliseconds
    pub latency_ms: f64,
}

impl QueryMetrics {
    /// Calculate metrics from query result and ground truth
    pub fn calculate(query: &BenchmarkQuery, result: &QueryResult) -> Self {
        let expected_set: std::collections::HashSet<_> = query.expected_ids.iter().collect();
        let returned_set: std::collections::HashSet<_> = result.returned_ids.iter().collect();

        let relevant_returned = returned_set.intersection(&expected_set).count();
        let total_returned = result.returned_ids.len();
        let total_relevant = query.expected_ids.len();

        let precision = if total_returned > 0 {
            relevant_returned as f64 / total_returned as f64
        } else {
            0.0
        };

        let recall = if total_relevant > 0 {
            relevant_returned as f64 / total_relevant as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };

        let irrelevant_returned = total_returned - relevant_returned;
        let hallucination_rate = if total_returned > 0 {
            irrelevant_returned as f64 / total_returned as f64
        } else {
            0.0
        };

        Self {
            query_id: query.id,
            precision,
            recall,
            f1_score,
            hallucination_rate,
            latency_ms: result.latency_ms,
        }
    }
}

/// Aggregated metrics across all queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateMetrics {
    /// System name
    pub system_name: String,
    /// Number of queries executed
    pub query_count: usize,
    /// Average precision
    pub avg_precision: f64,
    /// Average recall
    pub avg_recall: f64,
    /// Average F1 score
    pub avg_f1_score: f64,
    /// Average hallucination rate
    pub avg_hallucination_rate: f64,
    /// Latency P50 (median)
    pub latency_p50_ms: f64,
    /// Latency P95
    pub latency_p95_ms: f64,
    /// Latency P99
    pub latency_p99_ms: f64,
    /// Per-query metrics
    pub query_metrics: Vec<QueryMetrics>,
}

impl AggregateMetrics {
    /// Calculate aggregate metrics from individual query metrics
    pub fn from_query_metrics(system_name: impl Into<String>, metrics: Vec<QueryMetrics>) -> Self {
        let count = metrics.len();
        if count == 0 {
            return Self {
                system_name: system_name.into(),
                query_count: 0,
                avg_precision: 0.0,
                avg_recall: 0.0,
                avg_f1_score: 0.0,
                avg_hallucination_rate: 0.0,
                latency_p50_ms: 0.0,
                latency_p95_ms: 0.0,
                latency_p99_ms: 0.0,
                query_metrics: metrics,
            };
        }

        let avg_precision = metrics.iter().map(|m| m.precision).sum::<f64>() / count as f64;
        let avg_recall = metrics.iter().map(|m| m.recall).sum::<f64>() / count as f64;
        let avg_f1_score = metrics.iter().map(|m| m.f1_score).sum::<f64>() / count as f64;
        let avg_hallucination_rate = metrics.iter().map(|m| m.hallucination_rate).sum::<f64>() / count as f64;

        // Calculate latency percentiles
        let mut latencies: Vec<f64> = metrics.iter().map(|m| m.latency_ms).collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let latency_p50_ms = percentile(&latencies, 0.50);
        let latency_p95_ms = percentile(&latencies, 0.95);
        let latency_p99_ms = percentile(&latencies, 0.99);

        Self {
            system_name: system_name.into(),
            query_count: count,
            avg_precision,
            avg_recall,
            avg_f1_score,
            avg_hallucination_rate,
            latency_p50_ms,
            latency_p95_ms,
            latency_p99_ms,
            query_metrics: metrics,
        }
    }
}

/// Calculate percentile from sorted values
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (p * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Benchmark dataset containing queries with ground truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkDataset {
    /// Dataset name
    pub name: String,
    /// Description
    pub description: String,
    /// Version
    pub version: String,
    /// Queries with ground truth
    pub queries: Vec<BenchmarkQuery>,
}

impl BenchmarkDataset {
    /// Create a new empty dataset
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            version: "1.0.0".to_string(),
            queries: Vec::new(),
        }
    }

    /// Add a query to the dataset
    pub fn add_query(&mut self, query: BenchmarkQuery) {
        self.queries.push(query);
    }

    /// Number of queries
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_metrics_calculation() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let id4 = Uuid::new_v4();

        let query = BenchmarkQuery::new("test query", vec![id1, id2, id3]);
        let result = QueryResult::new(query.id, vec![id1, id2, id4], 10.5, 10);

        let metrics = QueryMetrics::calculate(&query, &result);

        // 2 relevant returned (id1, id2), 3 total returned, 3 total relevant
        assert!((metrics.precision - 2.0 / 3.0).abs() < 0.001);
        assert!((metrics.recall - 2.0 / 3.0).abs() < 0.001);
        // F1 = 2 * (2/3) * (2/3) / ((2/3) + (2/3)) = 2/3
        assert!((metrics.f1_score - 2.0 / 3.0).abs() < 0.001);
        // 1 irrelevant (id4), 3 returned
        assert!((metrics.hallucination_rate - 1.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        // P50 of 10 values (indices 0-9): 0.50 * 9 = 4.5 -> rounds to 5, value at index 5 = 6.0
        let p50 = percentile(&values, 0.50);
        assert!(p50 >= 5.0 && p50 <= 6.0, "P50 should be around 5-6, got {}", p50);
        // P95: 0.95 * 9 = 8.55 -> rounds to 9, value at index 9 = 10.0
        let p95 = percentile(&values, 0.95);
        assert!((p95 - 10.0).abs() < 0.5, "P95 should be around 10, got {}", p95);
    }

    #[test]
    fn test_aggregate_metrics() {
        let metrics = vec![
            QueryMetrics {
                query_id: Uuid::new_v4(),
                precision: 0.8,
                recall: 0.7,
                f1_score: 0.75,
                hallucination_rate: 0.2,
                latency_ms: 10.0,
            },
            QueryMetrics {
                query_id: Uuid::new_v4(),
                precision: 0.9,
                recall: 0.8,
                f1_score: 0.85,
                hallucination_rate: 0.1,
                latency_ms: 20.0,
            },
        ];

        let aggregate = AggregateMetrics::from_query_metrics("test", metrics);

        assert_eq!(aggregate.query_count, 2);
        assert!((aggregate.avg_precision - 0.85).abs() < 0.001);
        assert!((aggregate.avg_recall - 0.75).abs() < 0.001);
    }
}
