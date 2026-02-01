//! Feedback handler for RLKGF

use axum::{extract::State, http::StatusCode, Json};
use uuid::Uuid;

use crate::models::{ApiError, FeedbackRequest, FeedbackResponse, FeedbackType, UpdatedConfidences};
use crate::state::AppState;

/// POST /api/feedback - Submit feedback for an element
#[utoipa::path(
    post,
    path = "/api/feedback",
    request_body = FeedbackRequest,
    responses(
        (status = 200, description = "Feedback recorded", body = FeedbackResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 404, description = "Element not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "feedback"
)]
pub async fn submit_feedback(
    State(state): State<AppState>,
    Json(request): Json<FeedbackRequest>,
) -> Result<Json<FeedbackResponse>, (StatusCode, Json<ApiError>)> {
    // Find the element
    let element = state
        .repository
        .find_by_id(request.element_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", e.to_string())),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiError::new(
                    "not_found",
                    format!("Element {} not found", request.element_id),
                )),
            )
        })?;

    // Calculate updated confidences based on feedback
    // Using NARS revision formula approximation
    let is_positive = request.feedback_type == FeedbackType::ThumbsUp;
    let feedback_frequency = if is_positive { 1.0 } else { 0.0 };
    let feedback_confidence = 0.8; // High confidence in user feedback

    // Current values (default if not stored)
    let old_frequency = 0.9f32;
    let old_confidence = 0.5f32;

    // NARS revision: combine evidence
    let k = old_confidence / (1.0 - old_confidence + f32::EPSILON);
    let k_new = feedback_confidence / (1.0 - feedback_confidence + f32::EPSILON);
    let total_k = k + k_new;

    let new_frequency = (k * old_frequency + k_new * feedback_frequency) / total_k;
    let new_confidence = total_k / (total_k + 1.0);

    // Record metrics
    {
        let mut metrics = state.metrics.write().await;
        metrics.record_feedback(is_positive);
    }

    // TODO: Update element in graph with new confidence values
    // This would update the NARS truth values for this element

    let feedback_id = Uuid::new_v4();

    Ok(Json(FeedbackResponse {
        feedback_id,
        element_id: request.element_id,
        updated_confidences: UpdatedConfidences {
            old_frequency,
            new_frequency,
            old_confidence,
            new_confidence,
        },
    }))
}
