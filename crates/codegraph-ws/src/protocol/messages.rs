//! WebSocket message protocol types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Base message envelope for all WebSocket communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    /// Unique message ID for request/response correlation
    pub id: Uuid,
    /// Message type discriminator
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
    /// Type-specific payload
    pub payload: serde_json::Value,
}

impl WsMessage {
    pub fn new(msg_type: MessageType, payload: impl Serialize) -> Self {
        Self {
            id: Uuid::new_v4(),
            msg_type,
            timestamp: chrono::Utc::now().timestamp_millis(),
            payload: serde_json::to_value(payload).unwrap_or_default(),
        }
    }

    pub fn response(id: Uuid, msg_type: MessageType, payload: impl Serialize) -> Self {
        Self {
            id,
            msg_type,
            timestamp: chrono::Utc::now().timestamp_millis(),
            payload: serde_json::to_value(payload).unwrap_or_default(),
        }
    }

    pub fn error(id: Uuid, error: ErrorPayload) -> Self {
        Self::response(id, MessageType::Error, error)
    }
}

/// All supported message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    // Client -> Server
    ExtractRequest,
    QueryRequest,
    GenerateRequest,
    FeedbackSubmit,
    GraphStats,
    GraphElements,
    MetricsSubscribe,
    MetricsUnsubscribe,
    Ping,

    // Server -> Client
    ExtractProgress,
    ExtractComplete,
    QueryResult,
    GenerateStreaming,
    GenerateComplete,
    FeedbackAck,
    GraphStatsResult,
    GraphElementsResult,
    MetricsUpdate,
    Pong,
    Error,
}

// ============================================================================
// Extract Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRequest {
    pub html: String,
    #[serde(default)]
    pub css: Option<String>,
    #[serde(default)]
    pub js: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub design_system: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionPhase {
    Parsing,
    Detection,
    Ontology,
    Narsese,
    Embedding,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractProgress {
    pub phase: ExtractionPhase,
    pub progress: u8,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractComplete {
    pub snippet_id: Uuid,
    pub element_ids: Vec<Uuid>,
    pub narsese_statements: Vec<String>,
    pub design_system: String,
    pub processing_time_ms: u64,
}

// ============================================================================
// Query Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub design_system: Option<String>,
    #[serde(default)]
    pub include_reasoning: bool,
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub elements: Vec<ElementWithScore>,
    pub narsese_queries: Vec<String>,
    pub reasoning_explanation: Option<String>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementWithScore {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub design_system: String,
    pub score: f64,
    pub match_reason: String,
}

// ============================================================================
// Generate Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub query: String,
    #[serde(default)]
    pub design_system: Option<String>,
    #[serde(default = "default_true")]
    pub include_css: bool,
    #[serde(default = "default_true")]
    pub include_js: bool,
    #[serde(default)]
    pub use_references: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeSection {
    Html,
    Css,
    Javascript,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateStreaming {
    pub token: String,
    pub section: CodeSection,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateComplete {
    pub html: String,
    pub css: Option<String>,
    pub javascript: Option<String>,
    pub reference_elements: Vec<ElementWithScore>,
    pub narsese_reasoning: Vec<String>,
    pub generation_time_ms: u64,
}

// ============================================================================
// Feedback Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSubmit {
    pub element_id: Uuid,
    pub feedback_type: FeedbackType,
    #[serde(default)]
    pub query_context: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    ThumbsUp,
    ThumbsDown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackAck {
    pub feedback_id: Uuid,
    pub element_id: Uuid,
    pub new_confidence: f64,
}

// ============================================================================
// Graph Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatsRequest {
    // Empty for now, but can add filters later
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatsResult {
    pub total_nodes: u64,
    pub total_relationships: u64,
    pub nodes_by_label: Vec<LabelCount>,
    pub relationships_by_type: Vec<RelTypeCount>,
    pub avg_degree: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelCount {
    pub label: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelTypeCount {
    pub rel_type: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphElementsRequest {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub design_system: Option<String>,
}

fn default_page() -> usize {
    1
}

fn default_per_page() -> usize {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphElementsResult {
    pub elements: Vec<GraphElement>,
    pub total: u64,
    pub page: usize,
    pub per_page: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphElement {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub design_system: String,
    pub connections: u32,
}

// ============================================================================
// Metrics Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsUpdate {
    pub total_elements: u64,
    pub total_queries: u64,
    pub total_generations: u64,
    pub positive_feedback: u64,
    pub negative_feedback: u64,
    pub avg_query_latency_ms: f64,
    pub avg_generation_latency_ms: f64,
    pub elements_by_category: Vec<CategoryCount>,
    pub elements_by_design_system: Vec<DesignSystemCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSystemCount {
    pub design_system: String,
    pub count: u64,
}

// ============================================================================
// Error Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

impl ErrorPayload {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Serialize) -> Self {
        self.details = serde_json::to_value(details).ok();
        self
    }
}

// ============================================================================
// Common Error Codes
// ============================================================================

pub mod error_codes {
    pub const INVALID_MESSAGE: &str = "invalid_message";
    pub const PARSE_ERROR: &str = "parse_error";
    pub const NOT_FOUND: &str = "not_found";
    pub const EXTRACTION_FAILED: &str = "extraction_failed";
    pub const QUERY_FAILED: &str = "query_failed";
    pub const GENERATION_FAILED: &str = "generation_failed";
    pub const FEEDBACK_FAILED: &str = "feedback_failed";
    pub const TIMEOUT: &str = "timeout";
    pub const INTERNAL_ERROR: &str = "internal_error";
}
