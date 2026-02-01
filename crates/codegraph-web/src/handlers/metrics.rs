//! Metrics handler for RLKGF

use axum::{extract::State, http::StatusCode, Json};

use crate::models::{ApiError, CategoryCount, DesignSystemCount, RLKGFMetrics};
use crate::state::AppState;

/// GET /api/metrics - Get RLKGF metrics
#[utoipa::path(
    get,
    path = "/api/metrics",
    responses(
        (status = 200, description = "RLKGF metrics", body = RLKGFMetrics),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "metrics"
)]
pub async fn get_metrics(
    State(state): State<AppState>,
) -> Result<Json<RLKGFMetrics>, (StatusCode, Json<ApiError>)> {
    // Get total elements from repository
    let total_elements = state.repository.count().await.unwrap_or(0);

    // Get metrics from collector
    let metrics = state.metrics.read().await;

    // TODO: Get actual counts by category and design system from graph
    let elements_by_category = vec![
        CategoryCount {
            category: "button".to_string(),
            count: 0,
        },
        CategoryCount {
            category: "card".to_string(),
            count: 0,
        },
        CategoryCount {
            category: "form".to_string(),
            count: 0,
        },
    ];

    let elements_by_design_system = vec![
        DesignSystemCount {
            design_system: "tailwind".to_string(),
            count: 0,
        },
        DesignSystemCount {
            design_system: "bootstrap".to_string(),
            count: 0,
        },
        DesignSystemCount {
            design_system: "material-ui".to_string(),
            count: 0,
        },
    ];

    Ok(Json(RLKGFMetrics {
        total_elements,
        elements_by_category,
        elements_by_design_system,
        total_feedback: metrics.total_feedback,
        positive_feedback: metrics.positive_feedback,
        negative_feedback: metrics.negative_feedback,
        avg_query_latency_ms: metrics.avg_query_latency(),
        avg_generation_latency_ms: metrics.avg_generation_latency(),
        total_queries: metrics.total_queries,
        total_generations: metrics.total_generations,
    }))
}
