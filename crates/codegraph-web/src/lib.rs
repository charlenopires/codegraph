//! CodeGraph Web API - Axum HTTP server with health checks and metrics

pub mod handlers;
pub mod health;
pub mod middleware;
pub mod models;
pub mod prometheus;
pub mod routes;
pub mod server;
pub mod state;

pub use routes::{create_router, ApiDoc};
pub use server::serve;
pub use state::AppState;
