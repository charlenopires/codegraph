//! QdrantRepository - Vector storage operations
//!
//! Implements CRUD operations for embedding points in Qdrant collections.

use qdrant_client::qdrant::{
    Condition, CreateCollectionBuilder, Filter, HnswConfigDiff,
    PointId, PointStruct, Range, SearchPointsBuilder,
    UpsertPointsBuilder, VectorParamsBuilder,
    CreateFieldIndexCollectionBuilder, FieldType,
    DeletePointsBuilder,
};
use qdrant_client::Qdrant;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::collections::COLLECTIONS;
use crate::config::{QdrantConfig, VectorConfig};
use crate::error::{Result, VectorError};
use crate::models::{EmbeddingPoint, PointPayload, SearchFilter, SearchResult};

/// Repository for Qdrant vector operations
#[derive(Clone)]
pub struct QdrantRepository {
    /// Qdrant client
    client: Arc<Qdrant>,
    /// Vector configuration
    vector_config: VectorConfig,
}

impl QdrantRepository {
    /// Create a new QdrantRepository
    pub async fn new(config: QdrantConfig) -> Result<Self> {
        let client = Qdrant::from_url(&config.url)
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| VectorError::Connection(e.to_string()))?;

        Ok(Self {
            client: Arc::new(client),
            vector_config: config.vector_config,
        })
    }

    /// Initialize all collections
    #[instrument(skip(self))]
    pub async fn init_collections(&self) -> Result<()> {
        for collection_name in COLLECTIONS {
            self.create_collection(collection_name).await?;
        }

        info!("Initialized {} collections", COLLECTIONS.len());
        Ok(())
    }

    /// Create a collection with proper configuration
    #[instrument(skip(self))]
    pub async fn create_collection(&self, name: &str) -> Result<()> {
        // Check if collection exists
        let exists = self.client.collection_exists(name).await?;

        if exists {
            debug!(collection = name, "Collection already exists");
            return Ok(());
        }

        // Create collection with vector config
        let mut vectors_config = VectorParamsBuilder::new(
            self.vector_config.size,
            self.vector_config.distance.to_qdrant(),
        );

        if self.vector_config.on_disk {
            vectors_config = vectors_config.on_disk(true);
        }

        let mut create_builder = CreateCollectionBuilder::new(name)
            .vectors_config(vectors_config);

        // Add HNSW config if specified
        if let Some(hnsw) = &self.vector_config.hnsw_config {
            create_builder = create_builder.hnsw_config(
                HnswConfigDiff {
                    m: Some(hnsw.m),
                    ef_construct: Some(hnsw.ef_construct),
                    on_disk: Some(hnsw.on_disk),
                    ..Default::default()
                }
            );
        }

        self.client.create_collection(create_builder).await?;

        // Create payload indexes
        self.create_payload_indexes(name).await?;

        info!(collection = name, "Created collection");
        metrics::counter!("vector_collections_created").increment(1);

        Ok(())
    }

    /// Create payload indexes for efficient filtering
    async fn create_payload_indexes(&self, collection: &str) -> Result<()> {
        // Index on category (keyword)
        self.client
            .create_field_index(
                CreateFieldIndexCollectionBuilder::new(collection, "category", FieldType::Keyword)
            )
            .await?;

        // Index on element_type (keyword)
        self.client
            .create_field_index(
                CreateFieldIndexCollectionBuilder::new(collection, "element_type", FieldType::Keyword)
            )
            .await?;

        // Index on design_system (keyword)
        self.client
            .create_field_index(
                CreateFieldIndexCollectionBuilder::new(collection, "design_system", FieldType::Keyword)
            )
            .await?;

        // Index on confidence (float range)
        self.client
            .create_field_index(
                CreateFieldIndexCollectionBuilder::new(collection, "confidence", FieldType::Float)
            )
            .await?;

        debug!(collection = collection, "Created payload indexes");

        Ok(())
    }

    /// Upsert a single point
    #[instrument(skip(self, point))]
    pub async fn upsert_point(&self, collection: &str, point: EmbeddingPoint) -> Result<()> {
        // Validate vector dimension
        if point.vector.len() != self.vector_config.size as usize {
            return Err(VectorError::InvalidDimension {
                expected: self.vector_config.size as usize,
                actual: point.vector.len(),
            });
        }

        let point_struct = self.embedding_to_point_struct(&point)?;

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection, vec![point_struct]))
            .await?;

        debug!(collection = collection, id = %point.id, "Upserted point");
        metrics::counter!("vector_points_upserted").increment(1);

        Ok(())
    }

    /// Upsert multiple points in batch
    #[instrument(skip(self, points))]
    pub async fn upsert_batch(&self, collection: &str, points: Vec<EmbeddingPoint>) -> Result<usize> {
        if points.is_empty() {
            return Ok(0);
        }

        // Validate dimensions
        for point in &points {
            if point.vector.len() != self.vector_config.size as usize {
                return Err(VectorError::InvalidDimension {
                    expected: self.vector_config.size as usize,
                    actual: point.vector.len(),
                });
            }
        }

        let count = points.len();
        let point_structs: Vec<_> = points
            .into_iter()
            .filter_map(|p| self.embedding_to_point_struct(&p).ok())
            .collect();

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection, point_structs))
            .await?;

        info!(collection = collection, count = count, "Batch upserted points");
        metrics::counter!("vector_points_upserted").increment(count as u64);

        Ok(count)
    }

    /// Search for similar vectors
    #[instrument(skip(self, vector))]
    pub async fn search(
        &self,
        collection: &str,
        vector: Vec<f32>,
        limit: u64,
        filter: Option<SearchFilter>,
    ) -> Result<Vec<SearchResult>> {
        // Validate vector dimension
        if vector.len() != self.vector_config.size as usize {
            return Err(VectorError::InvalidDimension {
                expected: self.vector_config.size as usize,
                actual: vector.len(),
            });
        }

        let mut search_builder = SearchPointsBuilder::new(collection, vector, limit)
            .with_payload(true);

        // Apply filters
        if let Some(f) = filter {
            if f.is_active() {
                let qdrant_filter = self.build_filter(&f);
                search_builder = search_builder.filter(qdrant_filter);
            }
        }

        let response = self.client.search_points(search_builder).await?;

        let results: Vec<SearchResult> = response
            .result
            .into_iter()
            .filter_map(|p| self.scored_point_to_result(p))
            .collect();

        debug!(collection = collection, results = results.len(), "Search completed");
        metrics::counter!("vector_searches").increment(1);
        metrics::histogram!("vector_search_results").record(results.len() as f64);

        Ok(results)
    }

    /// Search across all collections
    #[instrument(skip(self, vector))]
    pub async fn search_all(
        &self,
        vector: Vec<f32>,
        limit: u64,
        filter: Option<SearchFilter>,
    ) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();

        for collection in COLLECTIONS {
            match self.search(collection, vector.clone(), limit, filter.clone()).await {
                Ok(results) => all_results.extend(results),
                Err(e) => {
                    warn!(collection = collection, error = %e, "Search failed in collection");
                }
            }
        }

        // Sort by score descending
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit total results
        all_results.truncate(limit as usize);

        Ok(all_results)
    }

    /// Delete a point by ID
    #[instrument(skip(self))]
    pub async fn delete_point(&self, collection: &str, id: Uuid) -> Result<()> {
        let point_id = PointId::from(id.to_string());

        self.client
            .delete_points(
                DeletePointsBuilder::new(collection)
                    .points(vec![point_id])
            )
            .await?;

        debug!(collection = collection, id = %id, "Deleted point");
        metrics::counter!("vector_points_deleted").increment(1);

        Ok(())
    }

    /// Delete multiple points
    #[instrument(skip(self, ids))]
    pub async fn delete_batch(&self, collection: &str, ids: Vec<Uuid>) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }

        let count = ids.len();
        let point_ids: Vec<_> = ids.into_iter().map(|id| PointId::from(id.to_string())).collect();

        self.client
            .delete_points(
                DeletePointsBuilder::new(collection)
                    .points(point_ids)
            )
            .await?;

        info!(collection = collection, count = count, "Batch deleted points");
        metrics::counter!("vector_points_deleted").increment(count as u64);

        Ok(count)
    }

    /// Get collection info
    pub async fn collection_info(&self, name: &str) -> Result<CollectionInfo> {
        let info = self.client.collection_info(name).await?;
        let result = info.result.ok_or_else(|| VectorError::CollectionNotFound(name.to_string()))?;

        Ok(CollectionInfo {
            name: name.to_string(),
            points_count: result.points_count.unwrap_or(0),
            // vectors_count not available in newer API, use points_count as estimate
            vectors_count: result.points_count.unwrap_or(0),
            indexed_vectors_count: result.indexed_vectors_count.unwrap_or(0),
        })
    }

    /// Get info for all collections
    pub async fn all_collections_info(&self) -> Result<Vec<CollectionInfo>> {
        let mut infos = Vec::new();

        for name in COLLECTIONS {
            match self.collection_info(name).await {
                Ok(info) => infos.push(info),
                Err(e) => {
                    warn!(collection = name, error = %e, "Failed to get collection info");
                }
            }
        }

        Ok(infos)
    }

    /// Convert EmbeddingPoint to Qdrant PointStruct
    fn embedding_to_point_struct(&self, point: &EmbeddingPoint) -> Result<PointStruct> {
        let id = PointId::from(point.id.to_string());

        let mut payload = std::collections::HashMap::new();
        payload.insert("name".to_string(), serde_json::json!(point.payload.name));
        payload.insert("category".to_string(), serde_json::json!(point.payload.category));
        payload.insert("element_type".to_string(), serde_json::json!(point.payload.element_type));
        payload.insert("design_system".to_string(), serde_json::json!(point.payload.design_system));
        payload.insert("confidence".to_string(), serde_json::json!(point.payload.confidence));
        payload.insert("css_classes".to_string(), serde_json::json!(point.payload.css_classes));
        payload.insert("tags".to_string(), serde_json::json!(point.payload.tags));

        Ok(PointStruct::new(id, point.vector.clone(), payload))
    }

    /// Convert Qdrant ScoredPoint to SearchResult
    fn scored_point_to_result(&self, point: qdrant_client::qdrant::ScoredPoint) -> Option<SearchResult> {
        let id_str = match point.id? {
            PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(s)) } => s,
            _ => return None,
        };

        let id = Uuid::parse_str(&id_str).ok()?;
        let payload = point.payload;

        let name = payload.get("name")?.as_str()?.to_string();
        let category = payload.get("category")?.as_str()?.to_string();
        let element_type = payload.get("element_type")?.as_str()?.to_string();
        let design_system = payload.get("design_system")?.as_str()?.to_string();
        let confidence = payload.get("confidence")?.as_double()? as f32;

        let css_classes: Vec<String> = payload
            .get("css_classes")
            .and_then(|v| v.as_list())
            .map(|l| l.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let tags: Vec<String> = payload
            .get("tags")
            .and_then(|v| v.as_list())
            .map(|l| l.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        Some(SearchResult {
            id,
            score: point.score,
            payload: PointPayload {
                name,
                category,
                element_type,
                design_system,
                confidence,
                css_classes,
                tags,
            },
        })
    }

    /// Build Qdrant filter from SearchFilter
    fn build_filter(&self, filter: &SearchFilter) -> Filter {
        let mut conditions = Vec::new();

        if let Some(ref category) = filter.category {
            conditions.push(Condition::matches("category", category.clone()));
        }

        if let Some(ref element_type) = filter.element_type {
            conditions.push(Condition::matches("element_type", element_type.clone()));
        }

        if let Some(ref design_system) = filter.design_system {
            conditions.push(Condition::matches("design_system", design_system.clone()));
        }

        if let Some(min_confidence) = filter.min_confidence {
            conditions.push(Condition::range(
                "confidence",
                Range {
                    gte: Some(min_confidence as f64),
                    ..Default::default()
                },
            ));
        }

        if let Some(ref tags) = filter.tags {
            for tag in tags {
                conditions.push(Condition::matches("tags", tag.clone()));
            }
        }

        Filter::must(conditions)
    }
}

/// Collection statistics
#[derive(Debug, Clone)]
pub struct CollectionInfo {
    /// Collection name
    pub name: String,
    /// Number of points
    pub points_count: u64,
    /// Number of vectors
    pub vectors_count: u64,
    /// Number of indexed vectors
    pub indexed_vectors_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_filter_empty() {
        let _config = QdrantConfig::default();
        // Can't test filter building without a repository instance
        // This is a placeholder for integration tests
    }

    #[test]
    fn test_search_filter_conditions() {
        let filter = SearchFilter::new()
            .with_category("button")
            .with_min_confidence(0.7);

        assert!(filter.is_active());
        assert_eq!(filter.category, Some("button".to_string()));
    }
}
