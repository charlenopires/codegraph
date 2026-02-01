//! Streaming utilities for real-time updates

use crate::protocol::*;
use tokio::sync::mpsc;

/// Progress reporter for extraction phases
pub struct ProgressReporter {
    sender: mpsc::Sender<WsMessage>,
    request_id: uuid::Uuid,
}

impl ProgressReporter {
    pub fn new(sender: mpsc::Sender<WsMessage>, request_id: uuid::Uuid) -> Self {
        Self { sender, request_id }
    }

    pub async fn report(&self, phase: ExtractionPhase, progress: u8, message: &str) {
        let msg = WsMessage::response(
            self.request_id,
            MessageType::ExtractProgress,
            ExtractProgress {
                phase,
                progress,
                message: message.to_string(),
            },
        );

        let _ = self.sender.send(msg).await;
    }
}

/// Token streamer for code generation
pub struct TokenStreamer {
    sender: mpsc::Sender<WsMessage>,
    request_id: uuid::Uuid,
}

impl TokenStreamer {
    pub fn new(sender: mpsc::Sender<WsMessage>, request_id: uuid::Uuid) -> Self {
        Self { sender, request_id }
    }

    pub async fn stream_token(&self, token: &str, section: CodeSection, is_complete: bool) {
        let msg = WsMessage::response(
            self.request_id,
            MessageType::GenerateStreaming,
            GenerateStreaming {
                token: token.to_string(),
                section,
                is_complete,
            },
        );

        let _ = self.sender.send(msg).await;
    }

    pub async fn stream_html(&self, token: &str) {
        self.stream_token(token, CodeSection::Html, false).await;
    }

    pub async fn stream_css(&self, token: &str) {
        self.stream_token(token, CodeSection::Css, false).await;
    }

    pub async fn stream_js(&self, token: &str) {
        self.stream_token(token, CodeSection::Javascript, false).await;
    }

    pub async fn complete(&self, section: CodeSection) {
        self.stream_token("", section, true).await;
    }
}
