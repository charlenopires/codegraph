//! Feedback handler - processes RLKGF feedback submissions

use crate::protocol::*;
use crate::state::SharedState;
use std::sync::Arc;
use tracing::info;

pub async fn handle_feedback(state: Arc<SharedState>, msg: WsMessage) -> Option<WsMessage> {
    let request: FeedbackSubmit = match serde_json::from_value(msg.payload.clone()) {
        Ok(req) => req,
        Err(e) => {
            return Some(WsMessage::error(
                msg.id,
                ErrorPayload::new(
                    error_codes::PARSE_ERROR,
                    format!("Invalid feedback request: {}", e),
                ),
            ));
        }
    };

    info!(
        "Processing feedback: element_id={}, type={:?}",
        request.element_id, request.feedback_type
    );

    // Calculate confidence delta based on feedback type
    let delta = match request.feedback_type {
        FeedbackType::ThumbsUp => 0.1,
        FeedbackType::ThumbsDown => -0.15,
    };

    // Record metrics
    let mut metrics = state.metrics.write().await;
    if request.feedback_type == FeedbackType::ThumbsUp {
        metrics.record_positive_feedback();
    } else {
        metrics.record_negative_feedback();
    }

    // Calculate new confidence (simplified - in production would update Neo4j)
    let new_confidence = 0.5 + delta;

    Some(WsMessage::response(
        msg.id,
        MessageType::FeedbackAck,
        FeedbackAck {
            feedback_id: uuid::Uuid::new_v4(),
            element_id: request.element_id,
            new_confidence,
        },
    ))
}
