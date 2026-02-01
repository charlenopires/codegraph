//! Middleware for the API

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::models::ApiError;

/// Rate limiting middleware using Redis
/// Limits: 100 requests per minute per IP
pub async fn rate_limit(
    State(redis): State<Option<Arc<redis::Client>>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiError>)> {
    const RATE_LIMIT: u64 = 100;
    const WINDOW_SECONDS: u64 = 60;

    let Some(redis_client) = redis else {
        // No Redis configured, skip rate limiting
        return Ok(next.run(request).await);
    };

    let ip = addr.ip().to_string();
    let key = format!("rate_limit:{}", ip);

    // Try to get connection
    let mut conn = match redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis connection failed, skipping rate limit: {}", e);
            return Ok(next.run(request).await);
        }
    };

    // Increment counter and get current value
    let result: Result<(u64, u64), redis::RedisError> = redis::pipe()
        .atomic()
        .incr(&key, 1u64)
        .expire(&key, WINDOW_SECONDS as i64)
        .query_async(&mut conn)
        .await;

    match result {
        Ok((count, _)) => {
            if count > RATE_LIMIT {
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiError::new(
                        "rate_limit_exceeded",
                        format!(
                            "Rate limit exceeded. Maximum {} requests per minute.",
                            RATE_LIMIT
                        ),
                    )),
                ));
            }
        }
        Err(e) => {
            tracing::warn!("Redis rate limit check failed: {}", e);
            // Continue without rate limiting on Redis error
        }
    }

    Ok(next.run(request).await)
}

/// Request ID middleware - adds a unique ID to each request and logs request/response
pub async fn request_id(request: Request<Body>, next: Next) -> Response {
    use std::time::Instant;

    let trace_id = uuid::Uuid::new_v4().to_string();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // Log request start
    tracing::info!(
        trace_id = %trace_id,
        method = %method,
        path = %path,
        "Request started"
    );

    let start = Instant::now();
    let mut response = next.run(request).await;
    let duration = start.elapsed();

    let status = response.status();

    // Add trace_id to response header
    response
        .headers_mut()
        .insert("X-Request-ID", trace_id.parse().unwrap());

    // Log request completion
    tracing::info!(
        trace_id = %trace_id,
        method = %method,
        path = %path,
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    response
}
