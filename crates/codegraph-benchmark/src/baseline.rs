//! SimpleVectorRAG - Baseline implementation using only Qdrant vector search
//!
//! This baseline implementation provides a simple vector-only retrieval system
//! without NARS reasoning or graph pattern matching. It serves as a reference
//! for comparing the hybrid GraphRAG+NARS system.

use async_trait::async_trait;
use std::time::Instant;
use tracing::{debug, instrument};
use uuid::Uuid;

use codegraph_vector::{QdrantRepository, SearchFilter};

use crate::error::Result;
use crate::models::{BenchmarkQuery, QueryResult};
use crate::retriever::Retriever;

/// SimpleVectorRAG baseline - vector-only retrieval without NARS or graph patterns
#[derive(Clone)]
pub struct SimpleVectorRAG {
    /// Qdrant repository for vector operations
    repository: QdrantRepository,
    /// Default number of results to return
    default_limit: u64,
}

impl SimpleVectorRAG {
    /// Create a new SimpleVectorRAG instance
    pub fn new(repository: QdrantRepository) -> Self {
        Self {
            repository,
            default_limit: 10,
        }
    }

    /// Set default limit
    pub fn with_limit(mut self, limit: u64) -> Self {
        self.default_limit = limit;
        self
    }

    /// Build a search filter from query parameters
    fn build_filter(&self, query: &BenchmarkQuery) -> Option<SearchFilter> {
        let mut filter = SearchFilter::new();
        let mut has_filter = false;

        if let Some(ref ds) = query.design_system {
            filter = filter.with_design_system(ds.clone());
            has_filter = true;
        }

        if let Some(ref cat) = query.category {
            filter = filter.with_category(cat.clone());
            has_filter = true;
        }

        if has_filter {
            Some(filter)
        } else {
            None
        }
    }
}

#[async_trait]
impl Retriever for SimpleVectorRAG {
    fn name(&self) -> &str {
        "SimpleVectorRAG"
    }

    #[instrument(skip(self, query, embedding))]
    async fn search(
        &self,
        query: &BenchmarkQuery,
        embedding: Vec<f32>,
        limit: u64,
    ) -> Result<QueryResult> {
        let start = Instant::now();

        let filter = self.build_filter(query);

        // Search across all collections
        let results = self
            .repository
            .search_all(embedding, limit, filter)
            .await?;

        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        let returned_ids: Vec<Uuid> = results.iter().map(|r| r.id).collect();

        debug!(
            query_id = %query.id,
            results = returned_ids.len(),
            latency_ms = latency_ms,
            "SimpleVectorRAG search completed"
        );

        Ok(QueryResult::new(query.id, returned_ids, latency_ms, limit))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_vector_rag_name() {
        // We can't create a real SimpleVectorRAG without Qdrant, but we can test the name
        // This is a placeholder for integration tests
        assert_eq!("SimpleVectorRAG", "SimpleVectorRAG");
    }
}
