//! Code generation handler

use axum::{extract::State, http::StatusCode, Json};
use std::time::Instant;
use uuid::Uuid;

use crate::models::{ApiError, GenerateRequest, GenerateResponse};
use crate::state::AppState;
use codegraph_generation::generator::GenerationRequest as GenRequest;
use codegraph_generation::prompt::SimilarElement;

/// POST /api/generate - Generate code from natural language
#[utoipa::path(
    post,
    path = "/api/generate",
    request_body = GenerateRequest,
    responses(
        (status = 200, description = "Generated code", body = GenerateResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "generate"
)]
pub async fn generate_code(
    State(state): State<AppState>,
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, (StatusCode, Json<ApiError>)> {
    let start = Instant::now();

    // Validate request
    if request.query.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("validation_error", "Query is required")),
        ));
    }

    // Get reference elements if requested
    let mut reference_elements = Vec::new();
    let mut narsese_reasoning = Vec::new();
    let mut similar_elements = Vec::new();
    let mut categories = Vec::new();

    if request.use_references {
        // Search for similar elements
        let retrieval_result = {
            let mut retriever = state.retriever.write().await;
            retriever.retrieve(&request.query).await.ok()
        };

        if let Some(result) = retrieval_result {
            for elem in &result.elements {
                reference_elements.push(Uuid::parse_str(&elem.element_id).unwrap_or_else(|_| Uuid::new_v4()));
                similar_elements.push(SimilarElement {
                    name: elem.name.clone(),
                    category: elem.category.clone(),
                    tags: elem.tags.clone(),
                    similarity: elem.semantic_similarity,
                });
                if !categories.contains(&elem.category) {
                    categories.push(elem.category.clone());
                }
            }
            narsese_reasoning = result
                .reasoning
                .input_statements
                .iter()
                .map(|s| s.statement.clone())
                .collect();
        }
    }

    // Build generation request
    let gen_request = GenRequest {
        description: request.query.clone(),
        similar_elements,
        reasoning: if narsese_reasoning.is_empty() {
            None
        } else {
            Some(narsese_reasoning.join("\n"))
        },
        categories,
    };

    // Generate code
    let generated = state.generator.generate(gen_request).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::new("generation_error", e.to_string())),
        )
    })?;

    let generation_time_ms = start.elapsed().as_millis() as u64;

    // Record metrics
    {
        let mut metrics = state.metrics.write().await;
        metrics.record_generation(generation_time_ms);
    }

    Ok(Json(GenerateResponse {
        html: generated.code.html.unwrap_or_default(),
        css: if request.include_css { generated.code.css } else { None },
        javascript: if request.include_js { generated.code.javascript } else { None },
        reference_elements,
        narsese_reasoning,
        generation_time_ms,
    }))
}
