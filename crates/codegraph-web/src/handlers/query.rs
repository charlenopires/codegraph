//! Query handler for natural language search

use axum::{extract::State, http::StatusCode, Json};
use std::time::Instant;
use uuid::Uuid;

use crate::models::{ApiError, ElementSummary, ElementWithScore, QueryRequest, QueryResponse};
use crate::state::AppState;

/// POST /api/query - Execute natural language query
#[utoipa::path(
    post,
    path = "/api/query",
    request_body = QueryRequest,
    responses(
        (status = 200, description = "Query results", body = QueryResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "query"
)]
pub async fn execute_query(
    State(state): State<AppState>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, (StatusCode, Json<ApiError>)> {
    let start = Instant::now();

    // Validate request
    if request.query.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("validation_error", "Query is required")),
        ));
    }

    // Execute hybrid retrieval
    let retrieval_result = {
        let mut retriever = state.retriever.write().await;
        retriever.retrieve(&request.query).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("retrieval_error", e.to_string())),
            )
        })?
    };

    // Convert results to response format
    let elements: Vec<ElementWithScore> = retrieval_result
        .elements
        .iter()
        .map(|r| ElementWithScore {
            element: ElementSummary {
                id: Uuid::parse_str(&r.element_id).unwrap_or_else(|_| Uuid::new_v4()),
                name: r.name.clone(),
                category: r.category.clone(),
                element_type: r.category.clone(), // Use category as element_type
                design_system: None,
                css_classes: vec![],
                tags: r.tags.clone(),
            },
            score: r.final_score,
            match_reason: format!("{:?} search with {:.2} confidence", r.source, r.narsese_confidence),
        })
        .collect();

    // Extract Narsese queries from reasoning
    let narsese_queries: Vec<String> = retrieval_result
        .reasoning
        .input_statements
        .iter()
        .map(|s| s.statement.clone())
        .collect();

    // Build reasoning explanation
    let reasoning_explanation = if request.include_reasoning {
        let derived: Vec<String> = retrieval_result
            .reasoning
            .derived_statements
            .iter()
            .map(|s| format!("{} (c={:.2})", s.statement, s.confidence))
            .collect();

        if derived.is_empty() {
            Some("No additional inferences derived".to_string())
        } else {
            Some(format!("Derived statements:\n{}", derived.join("\n")))
        }
    } else {
        None
    };

    let processing_time_ms = start.elapsed().as_millis() as u64;

    // Record metrics
    {
        let mut metrics = state.metrics.write().await;
        metrics.record_query(processing_time_ms);
    }

    Ok(Json(QueryResponse {
        elements,
        narsese_queries,
        reasoning_explanation,
        processing_time_ms,
    }))
}
