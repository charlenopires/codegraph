//! Prometheus metrics for the CodeGraph API

use axum::{body::Body, extract::MatchedPath, http::Request, middleware::Next, response::Response};
use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, Encoder, HistogramVec, TextEncoder,
};
use std::sync::OnceLock;
use std::time::Instant;

/// HTTP request counter by method, path, and status
static HTTP_REQUESTS_TOTAL: OnceLock<CounterVec> = OnceLock::new();

/// HTTP request duration histogram by method and path
static HTTP_REQUEST_DURATION: OnceLock<HistogramVec> = OnceLock::new();

/// Initialize Prometheus metrics (call once at startup)
pub fn init_metrics() {
    HTTP_REQUESTS_TOTAL.get_or_init(|| {
        register_counter_vec!(
            "codegraph_http_requests_total",
            "Total number of HTTP requests",
            &["method", "path", "status"]
        )
        .expect("Failed to register http_requests_total metric")
    });

    HTTP_REQUEST_DURATION.get_or_init(|| {
        register_histogram_vec!(
            "codegraph_http_request_duration_seconds",
            "HTTP request duration in seconds",
            &["method", "path"],
            vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
        )
        .expect("Failed to register http_request_duration metric")
    });
}

/// Middleware to collect Prometheus metrics for each request
pub async fn metrics_middleware(request: Request<Body>, next: Next) -> Response {
    let method = request.method().to_string();
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    let status = response.status().as_u16().to_string();

    // Record metrics
    if let Some(counter) = HTTP_REQUESTS_TOTAL.get() {
        counter
            .with_label_values(&[&method, &path, &status])
            .inc();
    }

    if let Some(histogram) = HTTP_REQUEST_DURATION.get() {
        histogram
            .with_label_values(&[&method, &path])
            .observe(duration.as_secs_f64());
    }

    response
}

/// Handler for /metrics endpoint - returns Prometheus text format
pub async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();

    encoder
        .encode(&metric_families, &mut buffer)
        .expect("Failed to encode metrics");

    String::from_utf8(buffer).expect("Metrics are not valid UTF-8")
}
