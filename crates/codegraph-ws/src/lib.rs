//! CodeGraph WebSocket API
//!
//! This crate provides a WebSocket server for real-time communication
//! with the CodeGraph frontend.

pub mod handlers;
pub mod protocol;
pub mod server;
pub mod state;
pub mod streaming;

pub use protocol::*;
pub use server::{create_router, serve};
pub use state::{MetricsCollector, SharedState};
