//! Extract handler - processes code snippet extraction

use crate::protocol::*;
use crate::state::SharedState;
use codegraph_extraction::ExtractionInput;
use std::sync::Arc;
use tracing::{error, info};

pub async fn handle_extract(state: Arc<SharedState>, msg: WsMessage) -> Option<WsMessage> {
    let request: ExtractRequest = match serde_json::from_value(msg.payload.clone()) {
        Ok(req) => req,
        Err(e) => {
            return Some(WsMessage::error(
                msg.id,
                ErrorPayload::new(
                    error_codes::PARSE_ERROR,
                    format!("Invalid extract request: {}", e),
                ),
            ));
        }
    };

    info!(
        "Processing extraction request: name={:?}, html_len={}",
        request.name,
        request.html.len()
    );

    let start = std::time::Instant::now();

    // Build extraction input
    let mut input = ExtractionInput::new(&request.html);
    if let Some(css) = &request.css {
        input = input.with_css(css);
    }
    if let Some(js) = &request.js {
        input = input.with_js(js);
    }

    // Run extraction pipeline
    let mut extraction = state.extraction.write().await;
    let result = match extraction.extract(input).await {
        Ok(result) => result,
        Err(e) => {
            error!("Extraction failed: {}", e);
            return Some(WsMessage::error(
                msg.id,
                ErrorPayload::new(error_codes::EXTRACTION_FAILED, e.to_string()),
            ));
        }
    };

    let snippet_id = uuid::Uuid::new_v4();

    // Generate element IDs based on ontology mapping
    let element_ids: Vec<uuid::Uuid> = result
        .ontology
        .elements
        .iter()
        .map(|_| uuid::Uuid::new_v4())
        .collect();

    // Get narsese statements as strings
    let narsese_statements: Vec<String> = result
        .narsese
        .statements
        .iter()
        .map(|s| s.statement.clone())
        .collect();

    let processing_time = start.elapsed().as_millis() as u64;

    Some(WsMessage::response(
        msg.id,
        MessageType::ExtractComplete,
        ExtractComplete {
            snippet_id,
            element_ids,
            narsese_statements,
            design_system: result.design_system.design_system.as_str().to_string(),
            processing_time_ms: processing_time,
        },
    ))
}
