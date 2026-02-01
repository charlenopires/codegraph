//! WebSocket message handlers

mod extract;
mod feedback;
mod generate;
mod graph;
mod metrics;
mod query;

pub use extract::*;
pub use feedback::*;
pub use generate::*;
pub use graph::*;
pub use metrics::*;
pub use query::*;

use crate::protocol::*;
use crate::state::SharedState;
use std::sync::Arc;

/// Trait for handling WebSocket messages
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, state: Arc<SharedState>, msg: WsMessage) -> Option<WsMessage>;
}

/// Route a message to the appropriate handler
pub async fn route_message(state: Arc<SharedState>, msg: WsMessage) -> Option<WsMessage> {
    match msg.msg_type {
        MessageType::ExtractRequest => handle_extract(state, msg).await,
        MessageType::QueryRequest => handle_query(state, msg).await,
        MessageType::GenerateRequest => handle_generate(state, msg).await,
        MessageType::FeedbackSubmit => handle_feedback(state, msg).await,
        MessageType::GraphStats => handle_graph_stats(state, msg).await,
        MessageType::GraphElements => handle_graph_elements(state, msg).await,
        MessageType::MetricsSubscribe => handle_metrics_subscribe(state, msg).await,
        MessageType::MetricsUnsubscribe => handle_metrics_unsubscribe(state, msg).await,
        MessageType::Ping => Some(WsMessage::response(msg.id, MessageType::Pong, ())),
        _ => Some(WsMessage::error(
            msg.id,
            ErrorPayload::new(
                error_codes::INVALID_MESSAGE,
                format!("Unsupported message type: {:?}", msg.msg_type),
            ),
        )),
    }
}
