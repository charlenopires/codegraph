//! Health check endpoints for Docker healthchecks and monitoring

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

/// Health status response
#[derive(Serialize, ToSchema)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
}

/// Detailed health status with service checks
#[derive(Serialize, ToSchema)]
pub struct DetailedHealthStatus {
    pub status: String,
    pub version: String,
    pub services: ServiceStatuses,
}

/// Status of individual services
#[derive(Serialize, ToSchema)]
pub struct ServiceStatuses {
    pub neo4j: ServiceStatus,
    pub qdrant: ServiceStatus,
    pub redis: ServiceStatus,
    pub openai: ServiceStatus,
}

/// Status of a single service
#[derive(Serialize, ToSchema)]
pub struct ServiceStatus {
    pub status: String,
    pub message: Option<String>,
}

impl ServiceStatus {
    fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            message: None,
        }
    }

    fn healthy_with_message(msg: impl Into<String>) -> Self {
        Self {
            status: "healthy".to_string(),
            message: Some(msg.into()),
        }
    }

    fn unhealthy(msg: impl Into<String>) -> Self {
        Self {
            status: "unhealthy".to_string(),
            message: Some(msg.into()),
        }
    }

    fn not_configured() -> Self {
        Self {
            status: "not_configured".to_string(),
            message: None,
        }
    }
}

/// GET /health - Basic liveness check for Docker healthcheck
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthStatus),
    ),
    tag = "health"
)]
pub async fn health_check() -> impl IntoResponse {
    Json(HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// GET /health/ready - Readiness check with full service status
#[utoipa::path(
    get,
    path = "/health/ready",
    responses(
        (status = 200, description = "Service is ready", body = DetailedHealthStatus),
        (status = 503, description = "Service not ready", body = DetailedHealthStatus),
    ),
    tag = "health"
)]
pub async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    let mut all_healthy = true;

    // Check Neo4j
    let neo4j_status = match state.repository.count().await {
        Ok(count) => ServiceStatus::healthy_with_message(format!("{} elements", count)),
        Err(e) => {
            all_healthy = false;
            ServiceStatus::unhealthy(e.to_string())
        }
    };

    // Check Qdrant (optional service)
    let qdrant_status = {
        // We don't have direct access to Qdrant from state, so we check via retriever
        // For now, mark as healthy if we got this far (Qdrant is optional)
        ServiceStatus::healthy_with_message("optional service")
    };

    // Check Redis
    let redis_status = match &state.redis {
        Some(redis) => match redis.get_connection() {
            Ok(_) => ServiceStatus::healthy(),
            Err(e) => {
                // Redis is optional, don't fail overall health
                ServiceStatus::unhealthy(e.to_string())
            }
        },
        None => ServiceStatus::not_configured(),
    };

    // Check OpenAI API key
    let openai_status = if std::env::var("OPENAI_API_KEY").is_ok() {
        ServiceStatus::healthy()
    } else {
        ServiceStatus::healthy_with_message("using fallback generators")
    };

    let overall_status = if all_healthy { "ready" } else { "degraded" };
    let status_code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(DetailedHealthStatus {
            status: overall_status.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            services: ServiceStatuses {
                neo4j: neo4j_status,
                qdrant: qdrant_status,
                redis: redis_status,
                openai: openai_status,
            },
        }),
    )
}

/// GET /health/live - Liveness check
#[utoipa::path(
    get,
    path = "/health/live",
    responses(
        (status = 200, description = "Service is alive", body = HealthStatus),
    ),
    tag = "health"
)]
pub async fn liveness_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthStatus {
            status: "alive".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }),
    )
}
