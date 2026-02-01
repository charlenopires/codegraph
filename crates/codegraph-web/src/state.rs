//! Application state shared across handlers

use std::sync::Arc;
use tokio::sync::RwLock;

use codegraph_extraction::ExtractionPipeline;
use codegraph_generation::VanillaCodeGenerator;
use codegraph_graph::Neo4jRepository;
use codegraph_retrieval::HybridRetriever;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Neo4j repository for graph operations
    pub repository: Arc<Neo4jRepository>,
    /// Hybrid retriever for search
    pub retriever: Arc<RwLock<HybridRetriever>>,
    /// Code generator
    pub generator: Arc<VanillaCodeGenerator>,
    /// Extraction pipeline
    pub extraction: Arc<RwLock<ExtractionPipeline>>,
    /// Redis client for rate limiting
    pub redis: Option<Arc<redis::Client>>,
    /// Metrics collector
    pub metrics: Arc<RwLock<MetricsCollector>>,
}

impl AppState {
    /// Create new application state with all dependencies
    pub async fn new(
        repository: Neo4jRepository,
        retriever: HybridRetriever,
        generator: VanillaCodeGenerator,
    ) -> anyhow::Result<Self> {
        let redis = match std::env::var("REDIS_URL") {
            Ok(url) => {
                let client = redis::Client::open(url)?;
                Some(Arc::new(client))
            }
            Err(_) => None,
        };

        Ok(Self {
            repository: Arc::new(repository),
            retriever: Arc::new(RwLock::new(retriever)),
            generator: Arc::new(generator),
            extraction: Arc::new(RwLock::new(ExtractionPipeline::new())),
            redis,
            metrics: Arc::new(RwLock::new(MetricsCollector::new())),
        })
    }
}

/// Metrics collector for RLKGF
#[derive(Debug, Default)]
pub struct MetricsCollector {
    pub total_queries: u64,
    pub total_generations: u64,
    pub total_feedback: u64,
    pub positive_feedback: u64,
    pub negative_feedback: u64,
    pub query_latencies: Vec<u64>,
    pub generation_latencies: Vec<u64>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_query(&mut self, latency_ms: u64) {
        self.total_queries += 1;
        self.query_latencies.push(latency_ms);
        // Keep only last 1000 samples
        if self.query_latencies.len() > 1000 {
            self.query_latencies.remove(0);
        }
    }

    pub fn record_generation(&mut self, latency_ms: u64) {
        self.total_generations += 1;
        self.generation_latencies.push(latency_ms);
        if self.generation_latencies.len() > 1000 {
            self.generation_latencies.remove(0);
        }
    }

    pub fn record_feedback(&mut self, positive: bool) {
        self.total_feedback += 1;
        if positive {
            self.positive_feedback += 1;
        } else {
            self.negative_feedback += 1;
        }
    }

    pub fn avg_query_latency(&self) -> f64 {
        if self.query_latencies.is_empty() {
            0.0
        } else {
            self.query_latencies.iter().sum::<u64>() as f64 / self.query_latencies.len() as f64
        }
    }

    pub fn avg_generation_latency(&self) -> f64 {
        if self.generation_latencies.is_empty() {
            0.0
        } else {
            self.generation_latencies.iter().sum::<u64>() as f64
                / self.generation_latencies.len() as f64
        }
    }
}
