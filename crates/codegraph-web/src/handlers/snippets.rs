//! Snippet CRUD handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::time::Instant;
use uuid::Uuid;

use crate::models::{
    ApiError, DeleteResponse, ListSnippetsQuery, ListSnippetsResponse, SnippetDetails,
    SnippetSummary, UploadSnippetRequest, UploadSnippetResponse,
};
use crate::state::AppState;
use codegraph_extraction::pipeline::ExtractionInput;

/// POST /api/snippets - Upload a code snippet
#[utoipa::path(
    post,
    path = "/api/snippets",
    request_body = UploadSnippetRequest,
    responses(
        (status = 201, description = "Snippet uploaded successfully", body = UploadSnippetResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "snippets"
)]
pub async fn upload_snippet(
    State(state): State<AppState>,
    Json(request): Json<UploadSnippetRequest>,
) -> Result<(StatusCode, Json<UploadSnippetResponse>), (StatusCode, Json<ApiError>)> {
    let start = Instant::now();

    // Validate request
    if request.html.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::new("validation_error", "HTML content is required")),
        ));
    }

    // Build extraction input
    let mut input = ExtractionInput::new(&request.html);
    if let Some(css) = &request.css {
        input = input.with_css(css);
    }
    if let Some(js) = &request.js {
        input = input.with_js(js);
    }

    // Run extraction pipeline
    let extraction_result = {
        let mut pipeline = state.extraction.write().await;
        pipeline.extract(input).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("extraction_error", e.to_string())),
            )
        })?
    };

    // Generate snippet ID
    let snippet_id = Uuid::new_v4();

    // Extract Narsese statements
    let narsese_statements: Vec<String> = extraction_result
        .narsese
        .statements
        .iter()
        .map(|s| s.statement.clone())
        .collect();

    // Get design system
    let design_system = if extraction_result.design_system.confidence > 0.3 {
        Some(extraction_result.design_system.design_system.as_str().to_string())
    } else {
        None
    };

    // Create UI elements in graph
    let mut element_ids = Vec::new();
    for element in &extraction_result.ontology.elements {
        let ui_element = codegraph_graph::UIElement {
            id: Uuid::new_v4(),
            name: request.name.clone().unwrap_or_else(|| element.element_type.clone()),
            category: element.category.as_str().to_string(),
            element_type: element.element_type.clone(),
            design_system: design_system.clone(),
            html_template: Some(request.html.clone()),
            css_classes: element.classes.clone(),
            tags: request.tags.clone(),
            embedding: extraction_result.embedding.as_ref().map(|e| e.embedding.clone()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        if let Err(e) = state.repository.save(&ui_element).await {
            tracing::warn!("Failed to save element: {}", e);
        } else {
            element_ids.push(ui_element.id);
        }
    }

    // Create Snippet node with HAS_ELEMENT relationships
    let snippet = codegraph_graph::Snippet::new(&request.html)
        .with_id(snippet_id)
        .with_element_ids(element_ids.clone());

    let snippet = if let Some(name) = &request.name {
        snippet.with_name(name)
    } else {
        snippet
    };

    let snippet = if let Some(css) = &request.css {
        snippet.with_css(css)
    } else {
        snippet
    };

    let snippet = if let Some(js) = &request.js {
        snippet.with_js(js)
    } else {
        snippet
    };

    let snippet = if let Some(ds) = &design_system {
        snippet.with_design_system(ds)
    } else {
        snippet
    };

    let snippet = snippet.with_tags(request.tags.clone());

    if let Err(e) = state.repository.save_snippet(&snippet).await {
        tracing::warn!("Failed to save snippet: {}", e);
    }

    let processing_time_ms = start.elapsed().as_millis() as u64;

    Ok((
        StatusCode::CREATED,
        Json(UploadSnippetResponse {
            snippet_id,
            element_ids,
            narsese_statements,
            design_system,
            processing_time_ms,
        }),
    ))
}

/// GET /api/snippets - List snippets with pagination
#[utoipa::path(
    get,
    path = "/api/snippets",
    params(ListSnippetsQuery),
    responses(
        (status = 200, description = "List of snippets", body = ListSnippetsResponse),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "snippets"
)]
pub async fn list_snippets(
    State(state): State<AppState>,
    Query(query): Query<ListSnippetsQuery>,
) -> Result<Json<ListSnippetsResponse>, (StatusCode, Json<ApiError>)> {
    // Fetch snippets from Neo4j with pagination and filters
    let (snippets, total) = state
        .repository
        .list_snippets(
            query.page,
            query.per_page,
            query.design_system.as_deref(),
            query.category.as_deref(),
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", e.to_string())),
            )
        })?;

    // Convert to API response format
    let snippets: Vec<SnippetSummary> = snippets
        .into_iter()
        .map(|s| SnippetSummary {
            id: s.id,
            name: s.name.unwrap_or_else(|| "Unnamed".to_string()),
            design_system: s.design_system,
            element_count: s.element_count as usize,
            tags: s.tags,
            created_at: s.created_at,
        })
        .collect();

    let total_pages = if query.per_page > 0 {
        ((total as f64) / (query.per_page as f64)).ceil() as u32
    } else {
        0
    };

    Ok(Json(ListSnippetsResponse {
        snippets,
        total,
        page: query.page,
        per_page: query.per_page,
        total_pages,
    }))
}

/// GET /api/snippets/:id - Get snippet details
#[utoipa::path(
    get,
    path = "/api/snippets/{id}",
    params(
        ("id" = Uuid, Path, description = "Snippet ID")
    ),
    responses(
        (status = 200, description = "Snippet details", body = SnippetDetails),
        (status = 404, description = "Snippet not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "snippets"
)]
pub async fn get_snippet(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SnippetDetails>, (StatusCode, Json<ApiError>)> {
    // Fetch snippet from Neo4j
    let snippet = state
        .repository
        .find_snippet_by_id(id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", e.to_string())),
            )
        })?;

    match snippet {
        Some(s) => {
            // Fetch element details for each element_id
            let mut elements = Vec::new();
            for element_id in &s.element_ids {
                if let Ok(Some(element)) = state.repository.find_by_id(*element_id).await {
                    elements.push(crate::models::ElementSummary {
                        id: element.id,
                        name: element.name,
                        category: element.category,
                        element_type: element.element_type,
                        design_system: element.design_system,
                        css_classes: element.css_classes,
                        tags: element.tags,
                    });
                }
            }

            Ok(Json(SnippetDetails {
                id: s.id,
                name: s.name.unwrap_or_else(|| "Unnamed".to_string()),
                html: s.html,
                css: s.css,
                js: s.js,
                design_system: s.design_system,
                elements,
                narsese_statements: vec![], // Would need to re-extract or store
                tags: s.tags,
                created_at: s.created_at,
                updated_at: s.updated_at,
            }))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("not_found", format!("Snippet {} not found", id))),
        )),
    }
}

/// DELETE /api/snippets/:id - Delete a snippet
#[utoipa::path(
    delete,
    path = "/api/snippets/{id}",
    params(
        ("id" = Uuid, Path, description = "Snippet ID")
    ),
    responses(
        (status = 200, description = "Snippet deleted", body = DeleteResponse),
        (status = 404, description = "Snippet not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "snippets"
)]
pub async fn delete_snippet(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteResponse>, (StatusCode, Json<ApiError>)> {
    // First, get the snippet to count elements
    let snippet = state
        .repository
        .find_snippet_by_id(id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("database_error", e.to_string())),
            )
        })?;

    match snippet {
        Some(s) => {
            let element_count = s.element_ids.len();

            // Delete snippet and orphaned elements
            let deleted = state
                .repository
                .delete_snippet(id, true) // delete_orphans = true
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiError::new("database_error", e.to_string())),
                    )
                })?;

            if deleted {
                Ok(Json(DeleteResponse {
                    deleted: true,
                    elements_removed: element_count,
                }))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiError::new("not_found", format!("Snippet {} not found", id))),
                ))
            }
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiError::new("not_found", format!("Snippet {} not found", id))),
        )),
    }
}
