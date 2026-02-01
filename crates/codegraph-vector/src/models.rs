//! Data models for vector operations

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An embedding point stored in Qdrant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingPoint {
    /// Unique identifier (element ID)
    pub id: Uuid,
    /// Embedding vector (1536 dimensions for OpenAI)
    pub vector: Vec<f32>,
    /// Payload metadata
    pub payload: PointPayload,
}

impl EmbeddingPoint {
    /// Create a new embedding point
    pub fn new(id: Uuid, vector: Vec<f32>, payload: PointPayload) -> Self {
        Self { id, vector, payload }
    }

    /// Validate vector dimensions
    pub fn validate(&self, expected_size: usize) -> bool {
        self.vector.len() == expected_size
    }
}

/// Payload metadata for an embedding point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointPayload {
    /// Element name
    pub name: String,
    /// Category (button, card, form, etc.)
    pub category: String,
    /// Element type (component, pattern, layout)
    pub element_type: String,
    /// Design system name
    pub design_system: String,
    /// NARS confidence value (0.0 - 1.0)
    pub confidence: f32,
    /// CSS classes used
    #[serde(default)]
    pub css_classes: Vec<String>,
    /// Additional tags
    #[serde(default)]
    pub tags: Vec<String>,
}

impl PointPayload {
    /// Create a new payload
    pub fn new(
        name: impl Into<String>,
        category: impl Into<String>,
        element_type: impl Into<String>,
        design_system: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            category: category.into(),
            element_type: element_type.into(),
            design_system: design_system.into(),
            confidence: 0.5,
            css_classes: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set CSS classes
    pub fn with_css_classes(mut self, classes: Vec<String>) -> Self {
        self.css_classes = classes;
        self
    }

    /// Set tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Search filter for vector queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilter {
    /// Filter by category
    pub category: Option<String>,
    /// Filter by element type
    pub element_type: Option<String>,
    /// Filter by design system
    pub design_system: Option<String>,
    /// Minimum confidence threshold
    pub min_confidence: Option<f32>,
    /// Filter by tags (any match)
    pub tags: Option<Vec<String>>,
}

impl SearchFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Filter by element type
    pub fn with_element_type(mut self, element_type: impl Into<String>) -> Self {
        self.element_type = Some(element_type.into());
        self
    }

    /// Filter by design system
    pub fn with_design_system(mut self, design_system: impl Into<String>) -> Self {
        self.design_system = Some(design_system.into());
        self
    }

    /// Filter by minimum confidence
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = Some(confidence);
        self
    }

    /// Filter by tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Check if any filter is active
    pub fn is_active(&self) -> bool {
        self.category.is_some()
            || self.element_type.is_some()
            || self.design_system.is_some()
            || self.min_confidence.is_some()
            || self.tags.is_some()
    }
}

/// Result from a similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Element ID
    pub id: Uuid,
    /// Similarity score (0.0 - 1.0 for cosine)
    pub score: f32,
    /// Payload metadata
    pub payload: PointPayload,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(id: Uuid, score: f32, payload: PointPayload) -> Self {
        Self { id, score, payload }
    }
}

/// Batch of points for bulk operations
#[derive(Debug, Clone)]
pub struct PointBatch {
    /// Points to upsert
    pub points: Vec<EmbeddingPoint>,
    /// Target collection
    pub collection: String,
}

impl PointBatch {
    /// Create a new batch
    pub fn new(collection: impl Into<String>) -> Self {
        Self {
            points: Vec::new(),
            collection: collection.into(),
        }
    }

    /// Add a point to the batch
    pub fn add(&mut self, point: EmbeddingPoint) {
        self.points.push(point);
    }

    /// Number of points in the batch
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_point_validation() {
        let point = EmbeddingPoint::new(
            Uuid::new_v4(),
            vec![0.0; 1536],
            PointPayload::new("Button", "button", "component", "tailwind"),
        );

        assert!(point.validate(1536));
        assert!(!point.validate(1024));
    }

    #[test]
    fn test_payload_builder() {
        let payload = PointPayload::new("Card", "card", "component", "material-ui")
            .with_confidence(0.85)
            .with_css_classes(vec!["MuiCard-root".to_string()])
            .with_tags(vec!["responsive".to_string()]);

        assert_eq!(payload.name, "Card");
        assert_eq!(payload.confidence, 0.85);
        assert_eq!(payload.css_classes.len(), 1);
        assert_eq!(payload.tags.len(), 1);
    }

    #[test]
    fn test_confidence_clamping() {
        let payload = PointPayload::new("Test", "test", "test", "custom")
            .with_confidence(1.5);
        assert_eq!(payload.confidence, 1.0);

        let payload = PointPayload::new("Test", "test", "test", "custom")
            .with_confidence(-0.5);
        assert_eq!(payload.confidence, 0.0);
    }

    #[test]
    fn test_search_filter_builder() {
        let filter = SearchFilter::new()
            .with_category("button")
            .with_min_confidence(0.7);

        assert!(filter.is_active());
        assert_eq!(filter.category, Some("button".to_string()));
        assert_eq!(filter.min_confidence, Some(0.7));
    }

    #[test]
    fn test_empty_filter() {
        let filter = SearchFilter::new();
        assert!(!filter.is_active());
    }

    #[test]
    fn test_point_batch() {
        let mut batch = PointBatch::new("ui_tailwind");
        assert!(batch.is_empty());

        batch.add(EmbeddingPoint::new(
            Uuid::new_v4(),
            vec![0.0; 1536],
            PointPayload::new("Button", "button", "component", "tailwind"),
        ));

        assert_eq!(batch.len(), 1);
        assert!(!batch.is_empty());
    }
}
