//! Template handlers for serving HTML pages
//!
//! Serves the web dashboard templates with HTMX support.

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;

use crate::state::AppState;

/// Include template at compile time
macro_rules! include_template {
    ($base:expr, $content:expr) => {{
        let base = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/templates/",
            $base
        ));
        let content = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/templates/",
            $content
        ));
        base.replace("{% block content %}{% endblock %}", content)
    }};
}

/// Render base template with content
fn render_page(content: &str, active_nav: &str) -> String {
    let base = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/base.html"
    ));

    // Replace content placeholder
    let html = base.replace("{% block content %}{% endblock %}", content);

    // Mark active nav item
    let html = match active_nav {
        "upload" => html
            .replace(r#"id="nav-upload" class="flex items-center"#,
                     r#"id="nav-upload" class="flex items-center bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300"#),
        "query" => html
            .replace(r#"id="nav-query" class="flex items-center"#,
                     r#"id="nav-query" class="flex items-center bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300"#),
        "graph" => html
            .replace(r#"id="nav-graph" class="flex items-center"#,
                     r#"id="nav-graph" class="flex items-center bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300"#),
        "metrics" => html
            .replace(r#"id="nav-metrics" class="flex items-center"#,
                     r#"id="nav-metrics" class="flex items-center bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300"#),
        _ => html,
    };

    html
}

/// Home page - redirects to upload
pub async fn home() -> Response {
    (
        StatusCode::TEMPORARY_REDIRECT,
        [(header::LOCATION, "/upload")],
    )
        .into_response()
}

/// Upload snippets page
pub async fn upload_page() -> Html<String> {
    let content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/upload.html"
    ));
    Html(render_page(content, "upload"))
}

/// Query and generate page
pub async fn query_page() -> Html<String> {
    let content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/query.html"
    ));
    Html(render_page(content, "query"))
}

/// Graph visualization page
pub async fn graph_page() -> Html<String> {
    let content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/graph.html"
    ));
    Html(render_page(content, "graph"))
}

/// Metrics dashboard page
pub async fn metrics_page() -> Html<String> {
    let content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/metrics.html"
    ));
    Html(render_page(content, "metrics"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_page_includes_content() {
        let content = "<div>Test Content</div>";
        let html = render_page(content, "upload");
        assert!(html.contains("Test Content"));
        assert!(html.contains("CodeGraph"));
    }

    #[tokio::test]
    async fn test_upload_page_returns_html() {
        let response = upload_page().await;
        let body = response.0;
        assert!(body.contains("Upload Code Snippet"));
    }

    #[tokio::test]
    async fn test_query_page_returns_html() {
        let response = query_page().await;
        let body = response.0;
        assert!(body.contains("Query"));
    }

    #[tokio::test]
    async fn test_graph_page_returns_html() {
        let response = graph_page().await;
        let body = response.0;
        assert!(body.contains("Graph"));
    }
}
