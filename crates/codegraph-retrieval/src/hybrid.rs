//! Hybrid retriever - combines vector, fulltext, and graph pattern matching
//!
//! Weights: vector similarity (40%) + fulltext (30%) + pattern matching (30%)
//!
//! ## Metrics Exposed
//!
//! - `retrieval_vector_latency_ms` - Vector search latency histogram
//! - `retrieval_fulltext_latency_ms` - Fulltext search latency histogram
//! - `retrieval_graph_latency_ms` - Graph search latency histogram
//! - `retrieval_total_latency_ms` - Total retrieval latency histogram
//! - `retrieval_results_count` - Number of results returned

use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use tracing::{debug, error, info, warn};

use codegraph_extraction::embedding::EmbeddingGenerator;
use codegraph_graph::Neo4jRepository;
use codegraph_reasoning::{ReasoningPipeline, ReasoningResult};
use codegraph_vector::{QdrantRepository, SearchResult};

use crate::query::{ProcessedQuery, QueryProcessor};
use crate::ranker::{Ranker, ResultSource, ScoredElement};

/// Hybrid search weights
#[derive(Debug, Clone, Copy)]
pub struct HybridWeights {
    pub vector_similarity: f32,
    pub fulltext: f32,
    pub pattern_matching: f32,
}

impl Default for HybridWeights {
    fn default() -> Self {
        Self {
            vector_similarity: 0.4,
            fulltext: 0.3,
            pattern_matching: 0.3,
        }
    }
}

/// Result of hybrid retrieval
#[derive(Debug)]
pub struct RetrievalResult {
    /// Original query
    pub query: String,
    /// Processed query information
    pub processed: ProcessedQuery,
    /// NARS reasoning result
    pub reasoning: ReasoningResult,
    /// Ranked and deduplicated results
    pub elements: Vec<ScoredElement>,
    /// Total retrieval time in milliseconds
    pub latency_ms: u64,
}

/// Hybrid retriever combining multiple search strategies
pub struct HybridRetriever {
    query_processor: QueryProcessor,
    reasoning_pipeline: ReasoningPipeline,
    ranker: Ranker,
    hybrid_weights: HybridWeights,
    max_results: usize,
    latency_target_ms: u64,
    /// Qdrant repository for vector searches
    qdrant_repository: Option<Arc<QdrantRepository>>,
    /// Neo4j repository for fulltext and graph searches
    neo4j_repository: Option<Arc<Neo4jRepository>>,
    /// Embedding generator for query vectorization
    embedding_generator: Arc<EmbeddingGenerator>,
}

impl HybridRetriever {
    pub fn new() -> Self {
        Self {
            query_processor: QueryProcessor::new(),
            reasoning_pipeline: ReasoningPipeline::new(),
            ranker: Ranker::new(),
            hybrid_weights: HybridWeights::default(),
            max_results: 10,
            latency_target_ms: 2000, // 2 seconds target
            qdrant_repository: None,
            neo4j_repository: None,
            embedding_generator: Arc::new(EmbeddingGenerator::new()),
        }
    }

    /// Create a new HybridRetriever with Qdrant repository for vector search
    pub fn with_qdrant(mut self, repository: Arc<QdrantRepository>) -> Self {
        self.qdrant_repository = Some(repository);
        self
    }

    /// Create a new HybridRetriever with Neo4j repository for fulltext and graph search
    pub fn with_neo4j(mut self, repository: Arc<Neo4jRepository>) -> Self {
        self.neo4j_repository = Some(repository);
        self
    }

    /// Set the embedding generator
    pub fn with_embedding_generator(mut self, generator: Arc<EmbeddingGenerator>) -> Self {
        self.embedding_generator = generator;
        self
    }

    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    pub fn with_hybrid_weights(mut self, weights: HybridWeights) -> Self {
        self.hybrid_weights = weights;
        self
    }

    /// Perform hybrid retrieval
    pub async fn retrieve(&mut self, query: &str) -> Result<RetrievalResult> {
        let start = Instant::now();
        info!("Starting hybrid retrieval for: {}", query);

        // Step 1: Process query to extract intent, components, attributes
        let processed = self.query_processor.process(query);
        debug!(
            "Query processed: intent={:?}, components={:?}",
            processed.intent, processed.component_types
        );

        // Step 2: Run NARS reasoning pipeline (with fallback to offline mode)
        let reasoning = match self.reasoning_pipeline.process(query) {
            Ok(r) => r,
            Err(e) => {
                warn!("NARS reasoning failed, using offline mode: {}", e);
                self.reasoning_pipeline.process_offline(query)
            }
        };
        debug!(
            "Reasoning complete: {} search terms",
            reasoning.search_terms.len()
        );

        // Step 3: Perform parallel searches (vector, fulltext, graph)
        // In production, these would be actual database calls
        let (vector_results, fulltext_results, graph_results) = tokio::join!(
            self.search_vector(&reasoning.search_terms),
            self.search_fulltext(&processed.search_terms),
            self.search_graph(&processed.component_types, &processed.attributes),
        );

        // Step 4: Combine results with hybrid weights
        let mut all_elements = Vec::new();

        for mut elem in vector_results.unwrap_or_default() {
            elem.semantic_similarity *= self.hybrid_weights.vector_similarity;
            all_elements.push(elem);
        }

        for mut elem in fulltext_results.unwrap_or_default() {
            elem.semantic_similarity *= self.hybrid_weights.fulltext;
            all_elements.push(elem);
        }

        for mut elem in graph_results.unwrap_or_default() {
            elem.semantic_similarity *= self.hybrid_weights.pattern_matching;
            all_elements.push(elem);
        }

        // Step 5: Apply NARS confidence to results
        for elem in &mut all_elements {
            // Find matching NARS statement confidence
            let nars_conf = reasoning
                .derived_statements
                .iter()
                .chain(reasoning.input_statements.iter())
                .filter(|s| {
                    s.statement.contains(&elem.category) || s.statement.contains(&elem.name)
                })
                .map(|s| s.confidence)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.5); // Default confidence

            elem.narsese_confidence = nars_conf;
        }

        // Step 6: Rank and deduplicate
        let ranked = self.ranker.rank_and_deduplicate(all_elements);

        // Step 7: Limit results
        let elements: Vec<_> = ranked.into_iter().take(self.max_results).collect();

        let latency_ms = start.elapsed().as_millis() as u64;

        // Record total retrieval latency and result count
        metrics::histogram!("retrieval_total_latency_ms").record(latency_ms as f64);
        metrics::histogram!("retrieval_results_count").record(elements.len() as f64);

        if latency_ms > self.latency_target_ms {
            warn!(
                "Retrieval latency {}ms exceeded target {}ms",
                latency_ms, self.latency_target_ms
            );
        }

        info!(
            "Hybrid retrieval complete: {} results in {}ms",
            elements.len(),
            latency_ms
        );

        Ok(RetrievalResult {
            query: query.to_string(),
            processed,
            reasoning,
            elements,
            latency_ms,
        })
    }

    /// Vector similarity search via Qdrant
    async fn search_vector(&self, terms: &[String]) -> Result<Vec<ScoredElement>> {
        let start = Instant::now();
        debug!("Vector search for terms: {:?}", terms);

        // Check if Qdrant repository is available
        let repository = match &self.qdrant_repository {
            Some(repo) => repo,
            None => {
                warn!("Qdrant repository not configured, skipping vector search");
                return Ok(vec![]);
            }
        };

        // Build query text from search terms
        let query_text = terms.join(" ");
        if query_text.is_empty() {
            return Ok(vec![]);
        }

        // Generate embedding for the query
        let embedding_result = match self.embedding_generator.generate_text_embedding(&query_text).await {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to generate embedding for query: {}", e);
                return Ok(vec![]);
            }
        };

        debug!(
            "Generated query embedding with {} dimensions using model '{}'",
            embedding_result.dimensions, embedding_result.model
        );

        // Search across all collections
        let search_results = match repository
            .search_all(
                embedding_result.embedding,
                self.max_results as u64,
                None, // No filter for now
            )
            .await
        {
            Ok(results) => results,
            Err(e) => {
                error!("Qdrant search failed: {}", e);
                return Ok(vec![]);
            }
        };

        // Record vector search latency
        let latency_ms = start.elapsed().as_millis() as f64;
        metrics::histogram!("retrieval_vector_latency_ms").record(latency_ms);
        debug!("Vector search returned {} results in {}ms", search_results.len(), latency_ms);

        // Convert SearchResult to ScoredElement
        let scored_elements: Vec<ScoredElement> = search_results
            .into_iter()
            .map(|result| self.search_result_to_scored_element(result))
            .collect();

        Ok(scored_elements)
    }

    /// Convert a Qdrant SearchResult to a ScoredElement
    fn search_result_to_scored_element(&self, result: SearchResult) -> ScoredElement {
        ScoredElement {
            element_id: result.id.to_string(),
            name: result.payload.name,
            category: result.payload.category,
            tags: result.payload.tags,
            narsese_confidence: result.payload.confidence,
            semantic_similarity: result.score,
            graph_degree: 0.0, // Will be updated by graph search if available
            final_score: 0.0,  // Will be calculated by ranker
            source: ResultSource::Vector,
        }
    }

    /// Fulltext search via Neo4j
    async fn search_fulltext(&self, terms: &[String]) -> Result<Vec<ScoredElement>> {
        let start = Instant::now();
        debug!("Fulltext search for terms: {:?}", terms);

        // Check if Neo4j repository is available
        let repository = match &self.neo4j_repository {
            Some(repo) => repo,
            None => {
                warn!("Neo4j repository not configured, skipping fulltext search");
                return Ok(vec![]);
            }
        };

        // Build search term from terms (Lucene query syntax)
        let search_term = if terms.is_empty() {
            return Ok(vec![]);
        } else if terms.len() == 1 {
            terms[0].clone()
        } else {
            // Combine terms with OR for broader matching
            terms.join(" OR ")
        };

        // Execute fulltext search
        let results = match repository.fulltext_search(&search_term, self.max_results).await {
            Ok(results) => results,
            Err(e) => {
                error!("Neo4j fulltext search failed: {}", e);
                return Ok(vec![]);
            }
        };

        // Record fulltext search latency
        let latency_ms = start.elapsed().as_millis() as f64;
        metrics::histogram!("retrieval_fulltext_latency_ms").record(latency_ms);
        debug!("Fulltext search returned {} results in {}ms", results.len(), latency_ms);

        // Convert SimilarElement to ScoredElement
        let scored_elements: Vec<ScoredElement> = results
            .into_iter()
            .map(|result| self.similar_element_to_scored_element(result, ResultSource::Fulltext))
            .collect();

        Ok(scored_elements)
    }

    /// Convert a Neo4j SimilarElement to a ScoredElement
    fn similar_element_to_scored_element(
        &self,
        result: codegraph_graph::entities::SimilarElement,
        source: ResultSource,
    ) -> ScoredElement {
        ScoredElement {
            element_id: result.element.id.to_string(),
            name: result.element.name,
            category: result.element.category,
            tags: result.element.tags,
            narsese_confidence: 0.5, // Default, will be updated by NARS reasoning
            semantic_similarity: result.similarity,
            graph_degree: 0.0, // Will be updated by graph search if available
            final_score: 0.0,  // Will be calculated by ranker
            source,
        }
    }

    /// Graph pattern matching via Neo4j Cypher
    async fn search_graph(
        &self,
        component_types: &[String],
        attributes: &[String],
    ) -> Result<Vec<ScoredElement>> {
        let start = Instant::now();
        debug!(
            "Graph search for components: {:?}, attributes: {:?}",
            component_types, attributes
        );

        // Check if Neo4j repository is available
        let repository = match &self.neo4j_repository {
            Some(repo) => repo,
            None => {
                warn!("Neo4j repository not configured, skipping graph search");
                return Ok(vec![]);
            }
        };

        if component_types.is_empty() && attributes.is_empty() {
            return Ok(vec![]);
        }

        let mut all_elements = Vec::new();

        // Search by component types (categories)
        for component_type in component_types {
            match repository.find_by_category(component_type).await {
                Ok(elements) => {
                    for element in elements {
                        // Get graph degree for connectivity scoring
                        let degree = repository
                            .relations()
                            .get_degree(element.id)
                            .await
                            .unwrap_or(0);

                        // Normalize degree to 0-1 range (assume max 50 connections)
                        let normalized_degree = (degree as f32 / 50.0).min(1.0);

                        let scored = ScoredElement {
                            element_id: element.id.to_string(),
                            name: element.name.clone(),
                            category: element.category.clone(),
                            tags: element.tags.clone(),
                            narsese_confidence: 0.5, // Default
                            semantic_similarity: 0.8, // High score for category match
                            graph_degree: normalized_degree,
                            final_score: 0.0,
                            source: ResultSource::Graph,
                        };
                        all_elements.push(scored);
                    }
                }
                Err(e) => {
                    warn!("Failed to search by category '{}': {}", component_type, e);
                }
            }
        }

        // Filter by attributes (tags) if provided
        if !attributes.is_empty() {
            all_elements.retain(|elem| {
                // Check if any attribute matches any tag
                attributes.iter().any(|attr| {
                    elem.tags.iter().any(|tag| {
                        tag.to_lowercase().contains(&attr.to_lowercase())
                            || attr.to_lowercase().contains(&tag.to_lowercase())
                    })
                })
            });
        }

        // Record graph search latency
        let latency_ms = start.elapsed().as_millis() as f64;
        metrics::histogram!("retrieval_graph_latency_ms").record(latency_ms);
        debug!("Graph search returned {} results in {}ms", all_elements.len(), latency_ms);

        // Limit results
        all_elements.truncate(self.max_results);

        Ok(all_elements)
    }
}

impl Default for HybridRetriever {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hybrid_retrieval() {
        let mut retriever = HybridRetriever::new();
        let result = retriever.retrieve("create a responsive button").await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.processed.intent, crate::query::Intent::Create);
        assert!(result.latency_ms < 5000); // Should complete in reasonable time
    }

    #[tokio::test]
    async fn test_search_vector_without_repository() {
        // When no Qdrant repository is configured, vector search should return empty
        let retriever = HybridRetriever::new();
        let terms = vec!["button".to_string(), "primary".to_string()];
        let result = retriever.search_vector(&terms).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_search_vector_with_empty_terms() {
        let retriever = HybridRetriever::new();
        let terms: Vec<String> = vec![];
        let result = retriever.search_vector(&terms).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_search_fulltext_without_repository() {
        // When no Neo4j repository is configured, fulltext search should return empty
        let retriever = HybridRetriever::new();
        let terms = vec!["button".to_string()];
        let result = retriever.search_fulltext(&terms).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_search_fulltext_with_empty_terms() {
        let retriever = HybridRetriever::new();
        let terms: Vec<String> = vec![];
        let result = retriever.search_fulltext(&terms).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_search_graph_without_repository() {
        // When no Neo4j repository is configured, graph search should return empty
        let retriever = HybridRetriever::new();
        let components = vec!["button".to_string()];
        let attributes = vec!["primary".to_string()];
        let result = retriever.search_graph(&components, &attributes).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_search_graph_with_empty_inputs() {
        let retriever = HybridRetriever::new();
        let components: Vec<String> = vec![];
        let attributes: Vec<String> = vec![];
        let result = retriever.search_graph(&components, &attributes).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_hybrid_weights_default() {
        let weights = HybridWeights::default();
        assert_eq!(weights.vector_similarity, 0.4);
        assert_eq!(weights.fulltext, 0.3);
        assert_eq!(weights.pattern_matching, 0.3);
    }

    #[tokio::test]
    async fn test_retriever_with_custom_weights() {
        let custom_weights = HybridWeights {
            vector_similarity: 0.5,
            fulltext: 0.25,
            pattern_matching: 0.25,
        };
        let retriever = HybridRetriever::new().with_hybrid_weights(custom_weights);
        assert_eq!(retriever.hybrid_weights.vector_similarity, 0.5);
    }

    #[tokio::test]
    async fn test_retriever_with_max_results() {
        let retriever = HybridRetriever::new().with_max_results(20);
        assert_eq!(retriever.max_results, 20);
    }

    #[tokio::test]
    async fn test_search_result_to_scored_element() {
        use codegraph_vector::models::PointPayload;
        use uuid::Uuid;

        let retriever = HybridRetriever::new();

        let search_result = SearchResult {
            id: Uuid::new_v4(),
            score: 0.95,
            payload: PointPayload::new("TestButton", "button", "component", "tailwind")
                .with_confidence(0.85)
                .with_tags(vec!["primary".to_string()]),
        };

        let scored = retriever.search_result_to_scored_element(search_result);

        assert_eq!(scored.name, "TestButton");
        assert_eq!(scored.category, "button");
        assert_eq!(scored.semantic_similarity, 0.95);
        assert_eq!(scored.narsese_confidence, 0.85);
        assert_eq!(scored.source, ResultSource::Vector);
    }
}
