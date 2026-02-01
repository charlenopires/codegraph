//! Shared application state for WebSocket server

use codegraph_extraction::ExtractionPipeline;
use codegraph_generation::VanillaCodeGenerator;
use codegraph_graph::Neo4jRepository;
use codegraph_retrieval::HybridRetriever;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state across all WebSocket connections
pub struct SharedState {
    /// Neo4j repository for graph operations
    pub repository: Arc<Neo4jRepository>,

    /// Hybrid retriever for vector+graph+NARS search
    pub retriever: Arc<RwLock<HybridRetriever>>,

    /// Code generator
    pub generator: Arc<VanillaCodeGenerator>,

    /// Extraction pipeline
    pub extraction: Arc<RwLock<ExtractionPipeline>>,

    /// Metrics collector
    pub metrics: Arc<RwLock<MetricsCollector>>,
}

impl SharedState {
    pub fn new(
        repository: Neo4jRepository,
        retriever: HybridRetriever,
        generator: VanillaCodeGenerator,
        extraction: ExtractionPipeline,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            retriever: Arc::new(RwLock::new(retriever)),
            generator: Arc::new(generator),
            extraction: Arc::new(RwLock::new(extraction)),
            metrics: Arc::new(RwLock::new(MetricsCollector::new())),
        }
    }
}

/// Metrics collector for tracking operations
pub struct MetricsCollector {
    pub total_queries: u64,
    pub total_generations: u64,
    pub positive_feedback: u64,
    pub negative_feedback: u64,
    query_latencies: Vec<f64>,
    generation_latencies: Vec<f64>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            total_queries: 0,
            total_generations: 0,
            positive_feedback: 0,
            negative_feedback: 0,
            query_latencies: Vec::with_capacity(1000),
            generation_latencies: Vec::with_capacity(1000),
        }
    }

    pub fn record_query(&mut self, latency_ms: f64) {
        self.total_queries += 1;
        if self.query_latencies.len() >= 1000 {
            self.query_latencies.remove(0);
        }
        self.query_latencies.push(latency_ms);
    }

    pub fn record_generation(&mut self, latency_ms: f64) {
        self.total_generations += 1;
        if self.generation_latencies.len() >= 1000 {
            self.generation_latencies.remove(0);
        }
        self.generation_latencies.push(latency_ms);
    }

    pub fn record_positive_feedback(&mut self) {
        self.positive_feedback += 1;
    }

    pub fn record_negative_feedback(&mut self) {
        self.negative_feedback += 1;
    }

    pub fn avg_query_latency(&self) -> f64 {
        if self.query_latencies.is_empty() {
            0.0
        } else {
            self.query_latencies.iter().sum::<f64>() / self.query_latencies.len() as f64
        }
    }

    pub fn avg_generation_latency(&self) -> f64 {
        if self.generation_latencies.is_empty() {
            0.0
        } else {
            self.generation_latencies.iter().sum::<f64>() / self.generation_latencies.len() as f64
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
