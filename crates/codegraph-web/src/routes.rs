//! Route configuration for the CodeGraph API

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{feedback, generate, graph, metrics, query, snippets, templates};
use crate::health;
use crate::middleware::{rate_limit, request_id};
use crate::prometheus::{metrics_handler, metrics_middleware};
use crate::state::AppState;

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        // Snippets
        snippets::upload_snippet,
        snippets::list_snippets,
        snippets::get_snippet,
        snippets::delete_snippet,
        // Query
        query::execute_query,
        // Generate
        generate::generate_code,
        // Feedback
        feedback::submit_feedback,
        // Metrics
        metrics::get_metrics,
        // Graph
        graph::list_elements,
        graph::get_graph_stats,
        // Health
        health::health_check,
    ),
    components(
        schemas(
            // Request/Response models
            crate::models::UploadSnippetRequest,
            crate::models::UploadSnippetResponse,
            crate::models::ListSnippetsQuery,
            crate::models::ListSnippetsResponse,
            crate::models::SnippetSummary,
            crate::models::SnippetDetails,
            crate::models::DeleteResponse,
            crate::models::QueryRequest,
            crate::models::QueryResponse,
            crate::models::ElementWithScore,
            crate::models::ElementSummary,
            crate::models::GenerateRequest,
            crate::models::GenerateResponse,
            crate::models::FeedbackRequest,
            crate::models::FeedbackResponse,
            crate::models::FeedbackType,
            crate::models::UpdatedConfidences,
            crate::models::RLKGFMetrics,
            crate::models::CategoryCount,
            crate::models::DesignSystemCount,
            crate::models::ListElementsQuery,
            crate::models::ListElementsResponse,
            crate::models::GraphStats,
            crate::models::LabelCount,
            crate::models::RelationshipTypeCount,
            crate::models::ApiError,
            // Health models
            crate::health::HealthStatus,
        )
    ),
    tags(
        (name = "snippets", description = "Code snippet CRUD operations"),
        (name = "query", description = "Natural language query endpoint"),
        (name = "generate", description = "Code generation endpoint"),
        (name = "feedback", description = "Feedback for RLKGF"),
        (name = "metrics", description = "System metrics"),
        (name = "graph", description = "Graph inspection"),
        (name = "health", description = "Health check endpoints"),
    ),
    info(
        title = "CodeGraph API",
        version = "0.1.0",
        description = "GraphRAG + NARS powered UI code generation API",
        license(name = "MIT"),
    )
)]
pub struct ApiDoc;

/// Build the main application router with all routes
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Extract Redis client for rate limiting middleware
    let redis_client = state.redis.clone();

    // API routes with rate limiting
    let api_routes = Router::new()
        // Snippet endpoints
        .route("/snippets", post(snippets::upload_snippet))
        .route("/snippets", get(snippets::list_snippets))
        .route("/snippets/{id}", get(snippets::get_snippet))
        .route("/snippets/{id}", delete(snippets::delete_snippet))
        // Query endpoint
        .route("/query", post(query::execute_query))
        // Generate endpoint
        .route("/generate", post(generate::generate_code))
        // Feedback endpoint
        .route("/feedback", post(feedback::submit_feedback))
        // Metrics endpoint
        .route("/metrics", get(metrics::get_metrics))
        // Graph endpoints
        .route("/graph/elements", get(graph::list_elements))
        .route("/graph/stats", get(graph::get_graph_stats))
        // Request ID middleware (adds X-Request-ID header)
        .layer(middleware::from_fn(request_id))
        // Rate limiting middleware (100 req/min per IP)
        .layer(middleware::from_fn_with_state(redis_client, rate_limit))
        // Add state to API routes
        .with_state(state.clone());

    // Build swagger UI - note: SwaggerUi returns Router<()>
    let swagger_router: Router<()> = SwaggerUi::new("/api/docs")
        .url("/api/openapi.json", ApiDoc::openapi())
        .into();

    Router::new()
        // Web dashboard pages (HTMX)
        .route("/", get(templates::home))
        .route("/upload", get(templates::upload_page))
        .route("/query", get(templates::query_page))
        .route("/graph", get(templates::graph_page))
        .route("/dashboard/metrics", get(templates::metrics_page))
        // Health endpoints (required for Docker healthchecks)
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
        // Prometheus metrics endpoint
        .route("/metrics", get(metrics_handler))
        // Add state to health routes
        .with_state(state)
        // API routes under /api prefix (already has state)
        .nest("/api", api_routes)
        // OpenAPI documentation - merge stateless router
        .merge(swagger_router)
        // Prometheus metrics middleware
        .layer(middleware::from_fn(metrics_middleware))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(cors)
}
