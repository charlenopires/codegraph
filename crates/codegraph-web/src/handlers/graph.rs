//! Graph inspection handlers

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};

use crate::models::{
    ApiError, ElementSummary, GraphStats, LabelCount, ListElementsQuery, ListElementsResponse,
    RelationshipTypeCount,
};
use crate::state::AppState;

/// GET /api/graph/elements - List graph elements with filters
#[utoipa::path(
    get,
    path = "/api/graph/elements",
    params(ListElementsQuery),
    responses(
        (status = 200, description = "List of elements", body = ListElementsResponse),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "graph"
)]
pub async fn list_elements(
    State(state): State<AppState>,
    Query(query): Query<ListElementsQuery>,
) -> Result<Json<ListElementsResponse>, (StatusCode, Json<ApiError>)> {
    // Get elements with optional category filter
    let elements = if let Some(category) = &query.category {
        state
            .repository
            .find_by_category(category)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError::new("database_error", e.to_string())),
                )
            })?
    } else {
        // Get all elements (limited for performance)
        state
            .repository
            .list_all_elements(query.per_page as usize * 5) // Fetch enough for pagination
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError::new("database_error", e.to_string())),
                )
            })?
    };

    let element_summaries: Vec<ElementSummary> = elements
        .iter()
        .map(|e| ElementSummary {
            id: e.id,
            name: e.name.clone(),
            category: e.category.clone(),
            element_type: e.element_type.clone(),
            design_system: e.design_system.clone(),
            css_classes: e.css_classes.clone(),
            tags: e.tags.clone(),
        })
        .collect();

    let total = element_summaries.len() as u64;
    let total_pages = if query.per_page > 0 {
        ((total as f64) / (query.per_page as f64)).ceil() as u32
    } else {
        0
    };

    // Apply pagination
    let start = ((query.page - 1) * query.per_page) as usize;
    let end = (start + query.per_page as usize).min(element_summaries.len());
    let paginated = if start < element_summaries.len() {
        element_summaries[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(Json(ListElementsResponse {
        elements: paginated,
        total,
        page: query.page,
        per_page: query.per_page,
        total_pages,
    }))
}

/// GET /api/graph/stats - Get graph statistics
#[utoipa::path(
    get,
    path = "/api/graph/stats",
    responses(
        (status = 200, description = "Graph statistics", body = GraphStats),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "graph"
)]
pub async fn get_graph_stats(
    State(state): State<AppState>,
) -> Result<Json<GraphStats>, (StatusCode, Json<ApiError>)> {
    // Get total node count
    let total_nodes = state.repository.count().await.unwrap_or(0);

    // Get actual relationship count from Neo4j
    let total_relationships = state.repository.count_relationships().await.unwrap_or(0);

    // Get nodes by label from Neo4j
    let label_counts = state.repository.count_by_label().await.unwrap_or_default();
    let nodes_by_label: Vec<LabelCount> = if label_counts.is_empty() {
        // Fallback to basic count if query fails
        vec![LabelCount {
            label: "UIElement".to_string(),
            count: total_nodes,
        }]
    } else {
        label_counts
            .into_iter()
            .map(|(label, count)| LabelCount { label, count })
            .collect()
    };

    // Get relationships by type from Neo4j
    let rel_counts = state.repository.count_relationships_by_type().await.unwrap_or_default();
    let relationships_by_type: Vec<RelationshipTypeCount> = rel_counts
        .into_iter()
        .map(|(relationship_type, count)| RelationshipTypeCount { relationship_type, count })
        .collect();

    // Calculate average degree (edges * 2 / nodes for undirected, edges / nodes for directed)
    let avg_degree = if total_nodes > 0 {
        (total_relationships as f64 * 2.0) / total_nodes as f64
    } else {
        0.0
    };

    Ok(Json(GraphStats {
        total_nodes,
        total_relationships,
        nodes_by_label,
        relationships_by_type,
        avg_degree,
    }))
}
