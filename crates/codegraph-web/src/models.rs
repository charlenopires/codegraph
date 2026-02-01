//! API request and response models

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

// ==================== Snippet Models ====================

/// Request to upload a code snippet
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UploadSnippetRequest {
    /// HTML content of the snippet
    pub html: String,
    /// Optional CSS content
    #[serde(default)]
    pub css: Option<String>,
    /// Optional JavaScript content
    #[serde(default)]
    pub js: Option<String>,
    /// Optional name for the snippet
    #[serde(default)]
    pub name: Option<String>,
    /// Optional tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Response after uploading a snippet
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UploadSnippetResponse {
    /// ID of the created snippet
    pub snippet_id: Uuid,
    /// IDs of extracted UI elements
    pub element_ids: Vec<Uuid>,
    /// Generated Narsese statements
    pub narsese_statements: Vec<String>,
    /// Detected design system
    pub design_system: Option<String>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Query parameters for listing snippets
#[derive(Debug, Clone, Deserialize, ToSchema, IntoParams)]
pub struct ListSnippetsQuery {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    /// Filter by design system (e.g., "tailwind", "bootstrap", "material")
    #[serde(default)]
    pub design_system: Option<String>,
    /// Filter by element category (e.g., "button", "form", "card")
    #[serde(default)]
    pub category: Option<String>,
    /// Filter by tag
    #[serde(default)]
    pub tag: Option<String>,
    /// Search in name
    #[serde(default)]
    pub search: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

/// Snippet summary for list view
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SnippetSummary {
    pub id: Uuid,
    pub name: String,
    pub design_system: Option<String>,
    pub element_count: usize,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Paginated response for snippets
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListSnippetsResponse {
    pub snippets: Vec<SnippetSummary>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Full snippet details
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SnippetDetails {
    pub id: Uuid,
    pub name: String,
    pub html: String,
    pub css: Option<String>,
    pub js: Option<String>,
    pub design_system: Option<String>,
    pub elements: Vec<ElementSummary>,
    pub narsese_statements: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Delete response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeleteResponse {
    pub deleted: bool,
    pub elements_removed: usize,
}

// ==================== Query Models ====================

/// Natural language query request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct QueryRequest {
    /// Natural language query
    pub query: String,
    /// Maximum number of results
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Filter by design system
    #[serde(default)]
    pub design_system: Option<String>,
    /// Include reasoning explanation
    #[serde(default = "default_true")]
    pub include_reasoning: bool,
}

fn default_limit() -> usize {
    10
}

fn default_true() -> bool {
    true
}

/// Query response with elements and reasoning
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct QueryResponse {
    /// Matched UI elements
    pub elements: Vec<ElementWithScore>,
    /// Narsese queries executed
    pub narsese_queries: Vec<String>,
    /// Reasoning explanation
    pub reasoning_explanation: Option<String>,
    /// Query processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Element with similarity/relevance score
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ElementWithScore {
    pub element: ElementSummary,
    pub score: f32,
    pub match_reason: String,
}

/// UI element summary
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ElementSummary {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub element_type: String,
    pub design_system: Option<String>,
    pub css_classes: Vec<String>,
    pub tags: Vec<String>,
}

// ==================== Generate Models ====================

/// Code generation request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct GenerateRequest {
    /// Natural language description of desired component
    pub query: String,
    /// Target design system
    #[serde(default)]
    pub design_system: Option<String>,
    /// Include CSS styles
    #[serde(default = "default_true")]
    pub include_css: bool,
    /// Include JavaScript
    #[serde(default)]
    pub include_js: bool,
    /// Use similar elements as reference
    #[serde(default = "default_true")]
    pub use_references: bool,
}

/// Generated code response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GenerateResponse {
    /// Generated HTML code
    pub html: String,
    /// Generated CSS code (if requested)
    pub css: Option<String>,
    /// Generated JavaScript code (if requested)
    pub javascript: Option<String>,
    /// Reference elements used for generation
    pub reference_elements: Vec<Uuid>,
    /// Narsese reasoning used
    pub narsese_reasoning: Vec<String>,
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
}

// ==================== Feedback Models ====================

/// Feedback request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct FeedbackRequest {
    /// Element ID receiving feedback
    pub element_id: Uuid,
    /// Feedback type: positive or negative
    pub feedback_type: FeedbackType,
    /// Optional query context
    #[serde(default)]
    pub query_context: Option<String>,
    /// Optional comment
    #[serde(default)]
    pub comment: Option<String>,
}

/// Feedback type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    ThumbsUp,
    ThumbsDown,
}

/// Feedback response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FeedbackResponse {
    pub feedback_id: Uuid,
    pub element_id: Uuid,
    /// Updated NARS confidence values
    pub updated_confidences: UpdatedConfidences,
}

/// Updated confidence values after feedback
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UpdatedConfidences {
    pub old_frequency: f32,
    pub new_frequency: f32,
    pub old_confidence: f32,
    pub new_confidence: f32,
}

// ==================== Metrics Models ====================

/// RLKGF metrics response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RLKGFMetrics {
    /// Total number of elements in the graph
    pub total_elements: u64,
    /// Elements by category
    pub elements_by_category: Vec<CategoryCount>,
    /// Elements by design system
    pub elements_by_design_system: Vec<DesignSystemCount>,
    /// Total feedback received
    pub total_feedback: u64,
    /// Positive feedback count
    pub positive_feedback: u64,
    /// Negative feedback count
    pub negative_feedback: u64,
    /// Average query latency in milliseconds
    pub avg_query_latency_ms: f64,
    /// Average generation latency in milliseconds
    pub avg_generation_latency_ms: f64,
    /// Total queries processed
    pub total_queries: u64,
    /// Total generations performed
    pub total_generations: u64,
}

/// Category count
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CategoryCount {
    pub category: String,
    pub count: u64,
}

/// Design system count
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DesignSystemCount {
    pub design_system: String,
    pub count: u64,
}

// ==================== Graph Models ====================

/// Query parameters for listing graph elements
#[derive(Debug, Clone, Deserialize, ToSchema, IntoParams)]
pub struct ListElementsQuery {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    /// Filter by category
    #[serde(default)]
    pub category: Option<String>,
    /// Filter by design system
    #[serde(default)]
    pub design_system: Option<String>,
    /// Filter by element type
    #[serde(default)]
    pub element_type: Option<String>,
}

/// Paginated response for graph elements
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListElementsResponse {
    pub elements: Vec<ElementSummary>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GraphStats {
    /// Total number of nodes
    pub total_nodes: u64,
    /// Total number of relationships
    pub total_relationships: u64,
    /// Node counts by label
    pub nodes_by_label: Vec<LabelCount>,
    /// Relationship counts by type
    pub relationships_by_type: Vec<RelationshipTypeCount>,
    /// Average degree (connections per node)
    pub avg_degree: f64,
}

/// Label count
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LabelCount {
    pub label: String,
    pub count: u64,
}

/// Relationship type count
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RelationshipTypeCount {
    pub relationship_type: String,
    pub count: u64,
}

// ==================== Error Models ====================

/// API error response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}
