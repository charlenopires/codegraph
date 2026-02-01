//! BenchmarkRunner - Executes benchmarks across retrieval systems
//!
//! Runs the same dataset against multiple retrieval systems and
//! collects comparable metrics.

use std::sync::Arc;
use tracing::{info, instrument};

use crate::error::Result;
use crate::models::{AggregateMetrics, BenchmarkDataset, QueryMetrics};
use crate::retriever::Retriever;

/// Embedding generator trait for converting query text to vectors
#[async_trait::async_trait]
pub trait EmbeddingGenerator: Send + Sync {
    /// Generate embedding for query text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

/// Mock embedding generator for testing
pub struct MockEmbedding {
    dimension: usize,
}

impl MockEmbedding {
    /// Create a mock embedding generator
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait::async_trait]
impl EmbeddingGenerator for MockEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Generate deterministic embeddings based on text hash
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        let embedding: Vec<f32> = (0..self.dimension)
            .map(|i| {
                let mut h = DefaultHasher::new();
                seed.hash(&mut h);
                i.hash(&mut h);
                let hash = h.finish();
                // Normalize to [-1, 1] range
                ((hash as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32
            })
            .collect();

        Ok(embedding)
    }
}

/// Benchmark runner that executes queries against retrieval systems
pub struct BenchmarkRunner<E: EmbeddingGenerator> {
    /// Embedding generator
    embedder: Arc<E>,
    /// Number of results to retrieve per query
    result_limit: u64,
}

impl<E: EmbeddingGenerator> BenchmarkRunner<E> {
    /// Create a new benchmark runner
    pub fn new(embedder: E) -> Self {
        Self {
            embedder: Arc::new(embedder),
            result_limit: 10,
        }
    }

    /// Set the result limit
    pub fn with_limit(mut self, limit: u64) -> Self {
        self.result_limit = limit;
        self
    }

    /// Run benchmark on a single retrieval system
    #[instrument(skip(self, retriever, dataset))]
    pub async fn run_single<R: Retriever>(
        &self,
        retriever: &R,
        dataset: &BenchmarkDataset,
    ) -> Result<AggregateMetrics> {
        info!(
            system = retriever.name(),
            queries = dataset.len(),
            "Starting benchmark run"
        );

        let mut query_metrics = Vec::with_capacity(dataset.len());

        for query in &dataset.queries {
            // Generate embedding for query
            let embedding = self.embedder.embed(&query.query).await?;

            // Execute search
            let result = retriever.search(query, embedding, self.result_limit).await?;

            // Calculate metrics
            let metrics = QueryMetrics::calculate(query, &result);
            query_metrics.push(metrics);
        }

        let aggregate = AggregateMetrics::from_query_metrics(retriever.name(), query_metrics);

        info!(
            system = retriever.name(),
            precision = format!("{:.3}", aggregate.avg_precision),
            recall = format!("{:.3}", aggregate.avg_recall),
            f1 = format!("{:.3}", aggregate.avg_f1_score),
            p50_ms = format!("{:.2}", aggregate.latency_p50_ms),
            "Benchmark completed"
        );

        Ok(aggregate)
    }

    /// Run benchmark comparing two systems
    #[instrument(skip(self, baseline, hybrid, dataset))]
    pub async fn run_comparison<B: Retriever, H: Retriever>(
        &self,
        baseline: &B,
        hybrid: &H,
        dataset: &BenchmarkDataset,
    ) -> Result<BenchmarkComparison> {
        info!("Running comparative benchmark");

        let baseline_metrics = self.run_single(baseline, dataset).await?;
        let hybrid_metrics = self.run_single(hybrid, dataset).await?;

        Ok(BenchmarkComparison {
            baseline: baseline_metrics,
            hybrid: hybrid_metrics,
        })
    }
}

/// Comparison results between baseline and hybrid systems
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkComparison {
    /// Baseline (SimpleVectorRAG) metrics
    pub baseline: AggregateMetrics,
    /// Hybrid (GraphRAG+NARS) metrics
    pub hybrid: AggregateMetrics,
}

impl BenchmarkComparison {
    /// Calculate improvement percentages
    pub fn improvements(&self) -> ImprovementMetrics {
        ImprovementMetrics {
            precision_improvement: calc_improvement(
                self.baseline.avg_precision,
                self.hybrid.avg_precision,
            ),
            recall_improvement: calc_improvement(
                self.baseline.avg_recall,
                self.hybrid.avg_recall,
            ),
            f1_improvement: calc_improvement(
                self.baseline.avg_f1_score,
                self.hybrid.avg_f1_score,
            ),
            hallucination_reduction: calc_improvement(
                self.hybrid.avg_hallucination_rate,  // Lower is better
                self.baseline.avg_hallucination_rate,
            ),
            latency_p50_change: calc_improvement(
                self.baseline.latency_p50_ms,
                self.hybrid.latency_p50_ms,
            ) * -1.0, // Negative means faster
        }
    }
}

/// Improvement percentages between baseline and hybrid
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImprovementMetrics {
    /// Precision improvement percentage
    pub precision_improvement: f64,
    /// Recall improvement percentage
    pub recall_improvement: f64,
    /// F1 score improvement percentage
    pub f1_improvement: f64,
    /// Hallucination rate reduction percentage
    pub hallucination_reduction: f64,
    /// Latency P50 change (negative = faster)
    pub latency_p50_change: f64,
}

fn calc_improvement(baseline: f64, improved: f64) -> f64 {
    if baseline == 0.0 {
        return 0.0;
    }
    ((improved - baseline) / baseline) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::generate_standard_dataset;
    use crate::retriever::GraphRAGRetriever;

    #[tokio::test]
    async fn test_mock_embedding() {
        let embedder = MockEmbedding::new(1536);
        let embedding = embedder.embed("test query").await.unwrap();
        assert_eq!(embedding.len(), 1536);

        // Check determinism
        let embedding2 = embedder.embed("test query").await.unwrap();
        assert_eq!(embedding, embedding2);
    }

    #[tokio::test]
    async fn test_benchmark_runner() {
        let embedder = MockEmbedding::new(1536);
        let runner = BenchmarkRunner::new(embedder);

        // Use a small subset of the dataset for testing
        let mut dataset = generate_standard_dataset();
        dataset.queries.truncate(5); // Only first 5 queries

        let retriever = GraphRAGRetriever::new();
        let metrics = runner.run_single(&retriever, &dataset).await.unwrap();

        assert_eq!(metrics.query_count, 5);
        assert_eq!(metrics.system_name, "GraphRAG+NARS");
    }

    #[test]
    fn test_calc_improvement() {
        assert!((calc_improvement(0.5, 0.75) - 50.0).abs() < 0.001);
        assert!((calc_improvement(0.8, 0.8) - 0.0).abs() < 0.001);
        assert!((calc_improvement(0.0, 0.5) - 0.0).abs() < 0.001);
    }
}
