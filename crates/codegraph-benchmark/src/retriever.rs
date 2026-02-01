//! Retriever trait for benchmarking different retrieval systems

use async_trait::async_trait;

use crate::error::Result;
use crate::models::{BenchmarkQuery, QueryResult};

/// Trait for retrieval systems that can be benchmarked
#[async_trait]
pub trait Retriever: Send + Sync {
    /// Get the name of this retriever system
    fn name(&self) -> &str;

    /// Search for similar elements given a query and its embedding
    ///
    /// # Arguments
    /// * `query` - The benchmark query with filters and expected results
    /// * `embedding` - The query embedding vector (1536 dimensions for OpenAI)
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// QueryResult containing returned IDs and latency
    async fn search(
        &self,
        query: &BenchmarkQuery,
        embedding: Vec<f32>,
        limit: u64,
    ) -> Result<QueryResult>;
}

/// Placeholder for the GraphRAG+NARS hybrid system
/// This will be implemented in a separate module using the actual graph and NARS integration
pub struct GraphRAGRetriever {
    name: String,
}

impl GraphRAGRetriever {
    /// Create a placeholder GraphRAG retriever
    /// In production, this would take the graph repository and NARS reasoner
    pub fn new() -> Self {
        Self {
            name: "GraphRAG+NARS".to_string(),
        }
    }
}

impl Default for GraphRAGRetriever {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Retriever for GraphRAGRetriever {
    fn name(&self) -> &str {
        &self.name
    }

    async fn search(
        &self,
        query: &BenchmarkQuery,
        _embedding: Vec<f32>,
        limit: u64,
    ) -> Result<QueryResult> {
        // Placeholder implementation - returns empty results
        // Will be replaced with actual graph traversal + NARS reasoning
        Ok(QueryResult::new(query.id, vec![], 0.0, limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphrag_name() {
        let retriever = GraphRAGRetriever::new();
        assert_eq!(retriever.name(), "GraphRAG+NARS");
    }
}
