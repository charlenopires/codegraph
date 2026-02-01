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

    // Get actual counts by category from Neo4j
    let category_counts = state.repository.count_by_category().await.unwrap_or_default();
    let elements_by_category: Vec<CategoryCount> = category_counts
        .into_iter()
        .map(|(category, count)| CategoryCount { category, count })
        .collect();

    // Get actual counts by design system from Neo4j
    let ds_counts = state.repository.count_by_design_system().await.unwrap_or_default();
    let elements_by_design_system: Vec<DesignSystemCount> = ds_counts
        .into_iter()
        .map(|(design_system, count)| DesignSystemCount { design_system, count })
        .collect();

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
