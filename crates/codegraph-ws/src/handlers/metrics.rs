//! Metrics handlers - processes metrics subscriptions and updates

use crate::protocol::*;
use crate::state::SharedState;
use std::sync::Arc;
use tracing::info;

pub async fn handle_metrics_subscribe(state: Arc<SharedState>, msg: WsMessage) -> Option<WsMessage> {
    info!("Processing metrics subscribe request");

    // Get current metrics snapshot
    let metrics_update = get_metrics_snapshot(&state).await;

    Some(WsMessage::response(
        msg.id,
        MessageType::MetricsUpdate,
        metrics_update,
    ))
}

pub async fn handle_metrics_unsubscribe(state: Arc<SharedState>, msg: WsMessage) -> Option<WsMessage> {
    info!("Processing metrics unsubscribe request");

    // For now, just acknowledge - subscription management would be handled at connection level
    Some(WsMessage::response(msg.id, MessageType::MetricsUpdate, ()))
}

/// Get current metrics snapshot from all sources
pub async fn get_metrics_snapshot(state: &SharedState) -> MetricsUpdate {
    let metrics = state.metrics.read().await;

    // Get total elements from repository
    let total_elements = state.repository.count().await.unwrap_or(0);

    // Get elements by category
    let elements_by_category = state
        .repository
        .count_by_category()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(category, count)| CategoryCount { category, count })
        .collect();

    // Get elements by design system
    let elements_by_design_system = state
        .repository
        .count_by_design_system()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(design_system, count)| DesignSystemCount {
            design_system,
            count,
        })
        .collect();

    MetricsUpdate {
        total_elements,
        total_queries: metrics.total_queries,
        total_generations: metrics.total_generations,
        positive_feedback: metrics.positive_feedback,
        negative_feedback: metrics.negative_feedback,
        avg_query_latency_ms: metrics.avg_query_latency(),
        avg_generation_latency_ms: metrics.avg_generation_latency(),
        elements_by_category,
        elements_by_design_system,
    }
}
