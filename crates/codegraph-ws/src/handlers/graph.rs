//! Graph handlers - processes graph statistics and element queries

use crate::protocol::*;
use crate::state::SharedState;
use std::sync::Arc;
use tracing::{error, info};

pub async fn handle_graph_stats(state: Arc<SharedState>, msg: WsMessage) -> Option<WsMessage> {
    info!("Processing graph stats request");

    // Get node counts
    let total_nodes = state.repository.count().await.unwrap_or(0);

    // Get relationship counts
    let total_relationships = state.repository.count_relationships().await.unwrap_or(0);

    // Get nodes by label
    let nodes_by_label_raw = state.repository.count_by_label().await.unwrap_or_default();
    let nodes_by_label: Vec<LabelCount> = nodes_by_label_raw
        .into_iter()
        .map(|(label, count)| LabelCount { label, count })
        .collect();

    // Get relationships by type
    let relationships_by_type_raw = state
        .repository
        .count_relationships_by_type()
        .await
        .unwrap_or_default();
    let relationships_by_type: Vec<RelTypeCount> = relationships_by_type_raw
        .into_iter()
        .map(|(rel_type, count)| RelTypeCount { rel_type, count })
        .collect();

    // Calculate average degree
    let avg_degree = if total_nodes > 0 {
        (total_relationships as f64 * 2.0) / total_nodes as f64
    } else {
        0.0
    };

    Some(WsMessage::response(
        msg.id,
        MessageType::GraphStatsResult,
        GraphStatsResult {
            total_nodes,
            total_relationships,
            nodes_by_label,
            relationships_by_type,
            avg_degree,
        },
    ))
}

pub async fn handle_graph_elements(state: Arc<SharedState>, msg: WsMessage) -> Option<WsMessage> {
    let request: GraphElementsRequest = match serde_json::from_value(msg.payload.clone()) {
        Ok(req) => req,
        Err(e) => {
            return Some(WsMessage::error(
                msg.id,
                ErrorPayload::new(
                    error_codes::PARSE_ERROR,
                    format!("Invalid graph elements request: {}", e),
                ),
            ));
        }
    };

    info!(
        "Processing graph elements: page={}, per_page={}, category={:?}",
        request.page, request.per_page, request.category
    );

    // Fetch elements using available method
    let elements_raw = match state.repository.list_all_elements(request.per_page).await {
        Ok(elements) => elements,
        Err(e) => {
            error!("Failed to fetch elements: {}", e);
            return Some(WsMessage::error(
                msg.id,
                ErrorPayload::new(error_codes::INTERNAL_ERROR, e.to_string()),
            ));
        }
    };

    let total = state.repository.count().await.unwrap_or(0);

    let elements: Vec<GraphElement> = elements_raw
        .into_iter()
        .filter(|e| {
            // Apply filters if present
            let category_match = request
                .category
                .as_ref()
                .map(|c| e.category == *c)
                .unwrap_or(true);
            let ds_match = request
                .design_system
                .as_ref()
                .map(|ds| e.design_system.as_ref().map(|eds| eds == ds).unwrap_or(false))
                .unwrap_or(true);
            category_match && ds_match
        })
        .map(|e| GraphElement {
            id: e.id,
            name: e.name,
            category: e.category,
            design_system: e.design_system.unwrap_or_default(),
            connections: 0, // Would need graph query to get this
        })
        .collect();

    Some(WsMessage::response(
        msg.id,
        MessageType::GraphElementsResult,
        GraphElementsResult {
            elements,
            total,
            page: request.page,
            per_page: request.per_page,
        },
    ))
}
