//! Integration Test: HTML upload → Extraction → Neo4j storage
//!
//! This test validates the complete flow from HTML snippet upload
//! through the extraction pipeline to Neo4j graph storage.
//!
//! Prerequisites:
//! - Neo4j running at localhost:7687
//! - Environment variables: NEO4J_URI, NEO4J_USER, NEO4J_PASSWORD
//!
//! Run with: cargo test --test test_upload_extraction_storage

use codegraph_extraction::pipeline::{ExtractionInput, ExtractionPipeline};
use codegraph_graph::{Neo4jRepository, UIElement};
use uuid::Uuid;

/// Test HTML snippet representing a Tailwind button
const TEST_HTML: &str = r#"
<button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
    Click me
</button>
"#;

/// Test HTML snippet representing a card component
const TEST_CARD_HTML: &str = r#"
<div class="max-w-sm rounded overflow-hidden shadow-lg">
    <div class="px-6 py-4">
        <div class="font-bold text-xl mb-2">Card Title</div>
        <p class="text-gray-700 text-base">
            Card content goes here.
        </p>
    </div>
</div>
"#;

#[tokio::test]
#[ignore = "requires Neo4j connection"]
async fn test_full_upload_extraction_storage_flow() {
    // Setup
    let repository = Neo4jRepository::new()
        .await
        .expect("Failed to connect to Neo4j - is it running?");

    // Initialize schema
    repository
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    let mut pipeline = ExtractionPipeline::new();

    // Step 1: Create extraction input from HTML
    let input = ExtractionInput::new(TEST_HTML);

    // Step 2: Run extraction pipeline
    let extraction_result = pipeline
        .extract(input)
        .await
        .expect("Extraction should succeed");

    // Verify extraction produced results
    assert!(
        !extraction_result.ontology.elements.is_empty(),
        "Should extract at least one UI element"
    );

    // Step 3: Create UIElement from extraction result
    let element = &extraction_result.ontology.elements[0];
    let element_id = Uuid::new_v4();

    let ui_element = UIElement {
        id: element_id,
        name: "Test Button".to_string(),
        category: element.category.as_str().to_string(),
        element_type: element.element_type.clone(),
        design_system: Some("tailwind".to_string()),
        html_template: Some(TEST_HTML.to_string()),
        css_classes: element.classes.clone(),
        tags: vec!["test".to_string(), "button".to_string()],
        embedding: extraction_result
            .embedding
            .as_ref()
            .map(|e| e.embedding.clone()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Step 4: Save to Neo4j
    repository
        .save(&ui_element)
        .await
        .expect("Failed to save element to Neo4j");

    // Step 5: Verify element exists in Neo4j
    let retrieved = repository
        .find_by_id(element_id)
        .await
        .expect("Query should succeed");

    assert!(retrieved.is_some(), "Element should be found in Neo4j");

    let retrieved_element = retrieved.unwrap();
    assert_eq!(retrieved_element.id, element_id);
    assert_eq!(retrieved_element.name, "Test Button");
    assert_eq!(retrieved_element.design_system, Some("tailwind".to_string()));

    // Cleanup
    repository
        .delete(element_id)
        .await
        .expect("Cleanup should succeed");
}

#[tokio::test]
#[ignore = "requires Neo4j connection"]
async fn test_extraction_detects_correct_element_category() {
    let mut pipeline = ExtractionPipeline::new();

    // Test button extraction
    let button_input = ExtractionInput::new(TEST_HTML);
    let button_result = pipeline.extract(button_input).await.unwrap();

    assert!(
        !button_result.ontology.elements.is_empty(),
        "Should extract button element"
    );
    let button_element = &button_result.ontology.elements[0];
    assert!(
        button_element.element_type.to_lowercase().contains("button")
            || button_element.category.as_str().to_lowercase().contains("button"),
        "Should detect as button type"
    );

    // Test card extraction
    let card_input = ExtractionInput::new(TEST_CARD_HTML);
    let card_result = pipeline.extract(card_input).await.unwrap();

    assert!(
        !card_result.ontology.elements.is_empty(),
        "Should extract card element"
    );
}

#[tokio::test]
#[ignore = "requires Neo4j connection"]
async fn test_design_system_detection() {
    let mut pipeline = ExtractionPipeline::new();

    // Test with Tailwind CSS classes
    let input = ExtractionInput::new(TEST_HTML);
    let result = pipeline.extract(input).await.unwrap();

    // Should detect Tailwind as design system
    assert!(
        result.design_system.confidence > 0.0,
        "Should have some design system confidence"
    );
}

#[tokio::test]
#[ignore = "requires Neo4j connection"]
async fn test_narsese_generation() {
    let mut pipeline = ExtractionPipeline::new();

    let input = ExtractionInput::new(TEST_HTML);
    let result = pipeline.extract(input).await.unwrap();

    // Should generate Narsese statements
    assert!(
        !result.narsese.statements.is_empty(),
        "Should generate Narsese statements"
    );

    // Verify statements have proper format
    for statement in &result.narsese.statements {
        assert!(
            !statement.statement.is_empty(),
            "Statement should not be empty"
        );
    }
}

#[tokio::test]
#[ignore = "requires Neo4j connection"]
async fn test_multiple_elements_storage() {
    let repository = Neo4jRepository::new()
        .await
        .expect("Failed to connect to Neo4j");

    repository.initialize_schema().await.ok();

    let mut pipeline = ExtractionPipeline::new();

    // Store multiple elements
    let mut stored_ids = Vec::new();

    for i in 0..3 {
        let html = format!(
            r#"<button class="btn-{} bg-blue-500">Button {}</button>"#,
            i, i
        );
        let input = ExtractionInput::new(&html);
        let result = pipeline.extract(input).await.unwrap();

        let element_id = Uuid::new_v4();
        let ui_element = UIElement {
            id: element_id,
            name: format!("Button {}", i),
            category: "button".to_string(),
            element_type: "button".to_string(),
            design_system: Some("tailwind".to_string()),
            html_template: Some(html),
            css_classes: vec![format!("btn-{}", i), "bg-blue-500".to_string()],
            tags: vec!["test".to_string()],
            embedding: result.embedding.as_ref().map(|e| e.embedding.clone()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        repository.save(&ui_element).await.unwrap();
        stored_ids.push(element_id);
    }

    // Verify all elements can be retrieved
    for id in &stored_ids {
        let element = repository.find_by_id(*id).await.unwrap();
        assert!(element.is_some(), "Each stored element should be retrievable");
    }

    // Verify category query works
    let buttons = repository.find_by_category("button").await.unwrap();
    assert!(
        buttons.len() >= 3,
        "Should find at least 3 buttons by category"
    );

    // Cleanup
    for id in stored_ids {
        repository.delete(id).await.ok();
    }
}

#[tokio::test]
async fn test_extraction_pipeline_without_database() {
    // This test runs without Neo4j to verify extraction alone
    let mut pipeline = ExtractionPipeline::new();

    let input = ExtractionInput::new(TEST_HTML);
    let result = pipeline.extract(input).await.unwrap();

    // Verify basic extraction works
    assert!(result.processing_time_ms > 0);
    assert!(!result.ontology.elements.is_empty());

    // Verify HTML was parsed
    assert!(!result.html.elements.is_empty());
}
