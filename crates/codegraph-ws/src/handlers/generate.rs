//! Generate handler - processes code generation requests with streaming

use crate::protocol::*;
use crate::state::SharedState;
use codegraph_generation::{GenerationRequest as GenRequest, SimilarElement as GenSimilarElement};
use std::sync::Arc;
use tracing::{error, info};

pub async fn handle_generate(state: Arc<SharedState>, msg: WsMessage) -> Option<WsMessage> {
    let request: GenerateRequest = match serde_json::from_value(msg.payload.clone()) {
        Ok(req) => req,
        Err(e) => {
            return Some(WsMessage::error(
                msg.id,
                ErrorPayload::new(error_codes::PARSE_ERROR, format!("Invalid generate request: {}", e)),
            ));
        }
    };

    info!(
        "Processing generation: query='{}', design_system={:?}",
        request.query, request.design_system
    );

    let start = std::time::Instant::now();

    // Optionally retrieve reference elements
    let reference_elements = if request.use_references {
        let mut retriever = state.retriever.write().await;
        match retriever.retrieve(&request.query).await {
            Ok(result) => result
                .elements
                .iter()
                .take(5)
                .map(|elem| ElementWithScore {
                    id: uuid::Uuid::parse_str(&elem.element_id).unwrap_or_default(),
                    name: elem.name.clone(),
                    category: elem.category.clone(),
                    design_system: String::new(),
                    score: elem.final_score as f64,
                    match_reason: "reference".to_string(),
                })
                .collect(),
            Err(e) => {
                error!("Failed to retrieve references: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // Generate code
    // TODO: Implement streaming generation
    // For now, we'll do synchronous generation
    let generation_request = GenRequest {
        description: request.query.clone(),
        similar_elements: reference_elements
            .iter()
            .map(|e| GenSimilarElement {
                name: e.name.clone(),
                category: e.category.clone(),
                tags: Vec::new(),
                similarity: e.score as f32,
            })
            .collect(),
        reasoning: None,
        categories: Vec::new(),
    };

    let result = match state.generator.generate(generation_request).await {
        Ok(result) => result,
        Err(e) => {
            error!("Generation failed: {}", e);
            return Some(WsMessage::error(
                msg.id,
                ErrorPayload::new(error_codes::GENERATION_FAILED, e.to_string()),
            ));
        }
    };

    let generation_time = start.elapsed().as_millis() as u64;

    Some(WsMessage::response(
        msg.id,
        MessageType::GenerateComplete,
        GenerateComplete {
            html: result.code.html.clone().unwrap_or_default(),
            css: if request.include_css {
                result.code.css.clone()
            } else {
                None
            },
            javascript: if request.include_js {
                result.code.javascript.clone()
            } else {
                None
            },
            reference_elements,
            narsese_reasoning: Vec::new(),
            generation_time_ms: generation_time,
        },
    ))
}
