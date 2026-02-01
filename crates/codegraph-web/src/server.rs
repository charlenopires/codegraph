//! Server initialization and startup

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

use crate::prometheus::init_metrics;
use crate::routes::create_router;
use crate::state::AppState;

/// Start the Axum web server with provided state
pub async fn serve(state: AppState, port: u16) -> anyhow::Result<()> {
    // Initialize Prometheus metrics
    init_metrics();

    let app = create_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Starting CodeGraph API on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown...");
        },
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown...");
        },
    }

    // Cleanup operations
    info!("Cleaning up connections...");
    // Note: Axum handles connection draining automatically during graceful shutdown
    // Redis/Neo4j connections are dropped when AppState goes out of scope
}
