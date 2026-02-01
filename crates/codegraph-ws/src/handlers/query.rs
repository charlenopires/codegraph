//! Query handler - processes natural language queries

use crate::protocol::*;
use crate::state::SharedState;
use std::sync::Arc;
use tracing::{error, info};

pub async fn handle_query(state: Arc<SharedState>, msg: WsMessage) -> Option<WsMessage> {
    let request: QueryRequest = match serde_json::from_value(msg.payload.clone()) {
        Ok(req) => req,
        Err(e) => {
            return Some(WsMessage::error(
                msg.id,
                ErrorPayload::new(error_codes::PARSE_ERROR, format!("Invalid query request: {}", e)),
            ));
        }
    };

    info!(
        "Processing query: query='{}', limit={}, design_system={:?}",
        request.query, request.limit, request.design_system
    );

    let start = std::time::Instant::now();

    // Execute hybrid retrieval
    let mut retriever = state.retriever.write().await;
    let result = match retriever.retrieve(&request.query).await {
        Ok(result) => result,
        Err(e) => {
            error!("Query failed: {}", e);
            return Some(WsMessage::error(
                msg.id,
                ErrorPayload::new(error_codes::QUERY_FAILED, e.to_string()),
            ));
        }
    };

    let processing_time = start.elapsed().as_millis() as u64;

    // Convert results to response format (limited by request.limit)
    let elements: Vec<ElementWithScore> = result
        .elements
        .iter()
        .take(request.limit)
        .map(|elem| ElementWithScore {
            id: uuid::Uuid::parse_str(&elem.element_id).unwrap_or_default(),
            name: elem.name.clone(),
            category: elem.category.clone(),
            design_system: String::new(), // ScoredElement doesn't have design_system
            score: elem.final_score as f64,
            match_reason: format!(
                "semantic: {:.2}, narsese: {:.2}, graph: {:.2}",
                elem.semantic_similarity, elem.narsese_confidence, elem.graph_degree
            ),
        })
        .collect();

    let reasoning_explanation = if request.include_reasoning {
        Some(format!(
            "Intent: {}\nDerived: {:?}",
            result.reasoning.intent,
            result
                .reasoning
                .derived_statements
                .iter()
                .map(|s| s.statement.clone())
                .collect::<Vec<_>>()
        ))
    } else {
        None
    };

    Some(WsMessage::response(
        msg.id,
        MessageType::QueryResult,
        QueryResult {
            elements,
            narsese_queries: result
                .reasoning
                .input_statements
                .iter()
                .map(|s| s.statement.clone())
                .collect(),
            reasoning_explanation,
            processing_time_ms: processing_time,
        },
    ))
}
