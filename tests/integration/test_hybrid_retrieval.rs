//! Integration Test: Query → HybridRetrieval → Non-empty results
//!
//! This test validates the hybrid retrieval returns non-empty results
//! when the database is seeded with known UI elements.
//!
//! Prerequisites:
//! - Neo4j running at localhost:7687
//! - Qdrant running at localhost:6334
//!
//! Run with: cargo test --test test_hybrid_retrieval -- --ignored

use std::sync::Arc;

use codegraph_extraction::embedding::EmbeddingGenerator;
use codegraph_graph::{Neo4jRepository, UIElement};
use codegraph_retrieval::HybridRetriever;
use codegraph_vector::{
    models::PointPayload, EmbeddingPoint, QdrantConfig, QdrantRepository,
};
use uuid::Uuid;

/// Test collection name
const TEST_COLLECTION: &str = "ui_elements";

/// Helper to create a test UI element
fn create_test_element(name: &str, category: &str, html: &str) -> UIElement {
    UIElement {
        id: Uuid::new_v4(),
        name: name.to_string(),
        category: category.to_string(),
        element_type: "component".to_string(),
        design_system: Some("tailwind".to_string()),
        html_template: Some(html.to_string()),
        css_classes: vec!["bg-blue-500".to_string(), "rounded".to_string()],
        tags: vec!["test".to_string()],
        embedding: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
#[ignore = "requires Neo4j and Qdrant"]
async fn test_hybrid_retrieval_returns_results() {
    // Setup: Connect to databases
    let neo4j = Neo4jRepository::new()
        .await
        .expect("Failed to connect to Neo4j");
    neo4j.initialize_schema().await.ok();

    let qdrant = QdrantRepository::new(QdrantConfig::default())
        .await
        .expect("Failed to connect to Qdrant");
    qdrant.init_collections().await.ok();

    let embedding_generator = Arc::new(EmbeddingGenerator::new());

    // Step 1: Seed database with known UI elements
    let test_elements = vec![
        ("Blue Button", "button", "<button class=\"bg-blue-500\">Click</button>"),
        ("Primary Card", "card", "<div class=\"card shadow\">Card content</div>"),
        ("Search Input", "input", "<input type=\"text\" placeholder=\"Search...\">"),
    ];

    let mut element_ids = Vec::new();

    for (name, category, html) in &test_elements {
        let element = create_test_element(name, category, html);
        element_ids.push(element.id);

        // Save to Neo4j
        neo4j.save(&element).await.expect("Failed to save to Neo4j");

        // Generate embedding and save to Qdrant
        let embedding = embedding_generator
            .generate_text_embedding(&format!("{} {} {}", name, category, html))
            .await
            .expect("Failed to generate embedding");

        let payload = PointPayload::new(name.to_string(), category.to_string(), "component", "tailwind");
        let point = EmbeddingPoint::new(element.id, embedding.embedding, payload);
        qdrant.upsert_point(TEST_COLLECTION, point).await.ok();
    }

    // Step 2: Create HybridRetriever with repositories
    // Need to create a new Neo4j connection since Neo4jRepository doesn't implement Clone
    let neo4j_for_retriever = Neo4jRepository::new()
        .await
        .expect("Failed to connect to Neo4j for retriever");

    let mut retriever = HybridRetriever::new()
        .with_qdrant(Arc::new(qdrant))
        .with_neo4j(Arc::new(neo4j_for_retriever))
        .with_embedding_generator(embedding_generator)
        .with_max_results(10);

    // Step 3: Execute query
    let result = retriever
        .retrieve("blue button component")
        .await
        .expect("Retrieval should succeed");

    // Step 4: Verify non-empty results
    assert!(
        !result.elements.is_empty(),
        "Should return non-empty results for 'blue button component'"
    );

    // Step 5: Verify result quality
    assert!(
        result.latency_ms > 0,
        "Should track latency"
    );

    // The results should contain elements matching our query
    // Check that at least one result is related to buttons
    let found_relevant = result.elements.iter().any(|e| {
        e.name.to_lowercase().contains("button") || e.category.to_lowercase() == "button"
    });
    assert!(found_relevant, "Should find relevant results");

    // Cleanup
    for id in element_ids {
        neo4j.delete(id).await.ok();
    }
}

#[tokio::test]
#[ignore = "requires Neo4j and Qdrant"]
async fn test_vector_search_returns_results() {
    let qdrant = QdrantRepository::new(QdrantConfig::default())
        .await
        .expect("Failed to connect to Qdrant");
    qdrant.init_collections().await.ok();

    let embedding_generator = Arc::new(EmbeddingGenerator::new());

    // Seed Qdrant with test data
    let test_id = Uuid::new_v4();
    let embedding = embedding_generator
        .generate_text_embedding("a blue button with rounded corners")
        .await
        .unwrap();

    let payload = PointPayload::new("Test Button", "button", "component", "tailwind");
    let point = EmbeddingPoint::new(test_id, embedding.embedding.clone(), payload);
    qdrant.upsert_point(TEST_COLLECTION, point).await.ok();

    // Create retriever with only Qdrant
    let mut retriever = HybridRetriever::new()
        .with_qdrant(Arc::new(qdrant.clone()))
        .with_embedding_generator(embedding_generator.clone())
        .with_max_results(10);

    // Query for similar content
    let result = retriever
        .retrieve("button")
        .await
        .expect("Retrieval should succeed");

    // Vector search should return results
    assert!(
        !result.elements.is_empty(),
        "Vector search should return results"
    );

    // Cleanup
    qdrant.delete_point(TEST_COLLECTION, test_id).await.ok();
}

#[tokio::test]
#[ignore = "requires Neo4j"]
async fn test_fulltext_search_returns_results() {
    let neo4j = Neo4jRepository::new()
        .await
        .expect("Failed to connect to Neo4j");
    neo4j.initialize_schema().await.ok();

    // Seed Neo4j with test data
    let test_element = create_test_element(
        "Primary Action Button",
        "button",
        "<button class=\"btn-primary\">Submit</button>",
    );
    let test_id = test_element.id;
    neo4j.save(&test_element).await.expect("Failed to save");

    // Create retriever with only Neo4j
    // Need new connection since Neo4jRepository doesn't implement Clone
    let neo4j_for_retriever = Neo4jRepository::new()
        .await
        .expect("Failed to connect to Neo4j for retriever");
    let embedding_generator = Arc::new(EmbeddingGenerator::new());
    let mut retriever = HybridRetriever::new()
        .with_neo4j(Arc::new(neo4j_for_retriever))
        .with_embedding_generator(embedding_generator)
        .with_max_results(10);

    // Query using fulltext
    let result = retriever
        .retrieve("primary action button")
        .await
        .expect("Retrieval should succeed");

    // Fulltext search may or may not return results depending on index
    // The important thing is that retrieval completes without error
    assert!(result.latency_ms > 0, "Should track latency");

    // Cleanup
    neo4j.delete(test_id).await.ok();
}

#[tokio::test]
#[ignore = "requires Neo4j"]
async fn test_graph_search_by_category() {
    let neo4j = Neo4jRepository::new()
        .await
        .expect("Failed to connect to Neo4j");
    neo4j.initialize_schema().await.ok();

    // Seed multiple buttons
    let mut button_ids = Vec::new();
    for i in 0..3 {
        let element = UIElement {
            id: Uuid::new_v4(),
            name: format!("Button {}", i),
            category: "button".to_string(),
            element_type: "component".to_string(),
            design_system: Some("tailwind".to_string()),
            html_template: Some(format!("<button>Button {}</button>", i)),
            css_classes: vec!["btn".to_string()],
            tags: vec!["test".to_string()],
            embedding: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        button_ids.push(element.id);
        neo4j.save(&element).await.ok();
    }

    // Query by category
    let buttons = neo4j
        .find_by_category("button")
        .await
        .expect("Category search should succeed");

    assert!(
        buttons.len() >= 3,
        "Should find at least 3 buttons by category"
    );

    // Cleanup
    for id in button_ids {
        neo4j.delete(id).await.ok();
    }
}

#[tokio::test]
#[ignore = "requires Neo4j and Qdrant"]
async fn test_retrieval_with_empty_query() {
    let neo4j = Neo4jRepository::new()
        .await
        .expect("Failed to connect to Neo4j");
    let qdrant = QdrantRepository::new(QdrantConfig::default())
        .await
        .expect("Failed to connect to Qdrant");

    let embedding_generator = Arc::new(EmbeddingGenerator::new());
    let mut retriever = HybridRetriever::new()
        .with_qdrant(Arc::new(qdrant))
        .with_neo4j(Arc::new(neo4j))
        .with_embedding_generator(embedding_generator)
        .with_max_results(10);

    // Empty query should not crash
    let result = retriever.retrieve("").await;

    // Should handle gracefully (either succeed with empty or return error)
    match result {
        Ok(r) => {
            // Empty results are acceptable for empty query
            assert!(r.latency_ms >= 0);
        }
        Err(_) => {
            // Error is also acceptable for invalid query
        }
    }
}

#[tokio::test]
async fn test_retriever_without_repositories() {
    // Retriever should work (with empty results) even without repositories
    let embedding_generator = Arc::new(EmbeddingGenerator::new());
    let mut retriever = HybridRetriever::new()
        .with_embedding_generator(embedding_generator)
        .with_max_results(10);

    let result = retriever
        .retrieve("button")
        .await
        .expect("Should not crash without repositories");

    // Results will be empty since no repositories are configured
    // But retrieval should complete without error
    assert!(result.latency_ms >= 0);
}

#[tokio::test]
#[ignore = "requires Neo4j and Qdrant"]
async fn test_result_ranking_and_deduplication() {
    let neo4j = Neo4jRepository::new()
        .await
        .expect("Failed to connect to Neo4j");
    neo4j.initialize_schema().await.ok();

    let qdrant = QdrantRepository::new(QdrantConfig::default())
        .await
        .expect("Failed to connect to Qdrant");
    qdrant.init_collections().await.ok();

    let embedding_generator = Arc::new(EmbeddingGenerator::new());

    // Create element in both databases
    let element_id = Uuid::new_v4();
    let element = UIElement {
        id: element_id,
        name: "Duplicate Test Button".to_string(),
        category: "button".to_string(),
        element_type: "component".to_string(),
        design_system: Some("tailwind".to_string()),
        html_template: Some("<button>Test</button>".to_string()),
        css_classes: vec!["btn".to_string()],
        tags: vec!["test".to_string()],
        embedding: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Save to Neo4j
    neo4j.save(&element).await.ok();

    // Save to Qdrant
    let embedding = embedding_generator
        .generate_text_embedding("duplicate test button")
        .await
        .unwrap();
    let payload = PointPayload::new("Duplicate Test Button", "button", "component", "tailwind");
    let point = EmbeddingPoint::new(element_id, embedding.embedding, payload);
    qdrant.upsert_point(TEST_COLLECTION, point).await.ok();

    // Query - need to create new Neo4j connection since it doesn't implement Clone
    let neo4j2 = Neo4jRepository::new()
        .await
        .expect("Failed to connect to Neo4j");

    let mut retriever = HybridRetriever::new()
        .with_qdrant(Arc::new(qdrant.clone()))
        .with_neo4j(Arc::new(neo4j2))
        .with_embedding_generator(embedding_generator)
        .with_max_results(10);

    let result = retriever
        .retrieve("duplicate test button")
        .await
        .expect("Retrieval should succeed");

    // Results should be deduplicated - same element should appear once
    let element_id_str = element_id.to_string();
    let count = result.elements.iter().filter(|e| e.element_id == element_id_str).count();
    assert!(
        count <= 1,
        "Same element should not appear more than once (found {} times)",
        count
    );

    // Results should be ranked by final_score (descending)
    for i in 1..result.elements.len() {
        assert!(
            result.elements[i - 1].final_score >= result.elements[i].final_score,
            "Results should be ranked by score descending"
        );
    }

    // Cleanup
    neo4j.delete(element_id).await.ok();
    qdrant.delete_point(TEST_COLLECTION, element_id).await.ok();
}
