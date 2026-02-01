//! Integration Test: Embedding generation → Qdrant storage
//!
//! This test validates embedding generation and vector storage in Qdrant.
//!
//! Prerequisites:
//! - Qdrant running at localhost:6334
//! - Optional: OPENAI_API_KEY for real embeddings
//!
//! Run with: cargo test --test test_embedding_qdrant_storage -- --ignored

use codegraph_extraction::embedding::EmbeddingGenerator;
use codegraph_vector::{
    EmbeddingPoint, QdrantConfig, QdrantRepository, SearchFilter,
    models::PointPayload,
};
use uuid::Uuid;

/// Expected embedding dimensions
const EMBEDDING_DIMENSIONS: usize = 1536;

/// Test collection name
const TEST_COLLECTION: &str = "ui_elements";

#[tokio::test]
#[ignore = "requires Qdrant connection"]
async fn test_full_embedding_to_qdrant_flow() {
    // Setup
    let qdrant = QdrantRepository::new(QdrantConfig::default())
        .await
        .expect("Failed to connect to Qdrant - is it running?");

    // Initialize collections
    qdrant
        .init_collections()
        .await
        .expect("Failed to initialize collections");

    let embedding_generator = EmbeddingGenerator::new();

    // Step 1: Generate embedding from text
    let test_text = "A blue button with rounded corners and hover effect for user interaction";
    let embedding_result = embedding_generator
        .generate_text_embedding(test_text)
        .await
        .expect("Embedding generation should succeed");

    // Step 2: Verify embedding dimensions
    assert_eq!(
        embedding_result.dimensions, EMBEDDING_DIMENSIONS,
        "Embedding should have {} dimensions",
        EMBEDDING_DIMENSIONS
    );
    assert_eq!(
        embedding_result.embedding.len(),
        EMBEDDING_DIMENSIONS,
        "Embedding vector should have correct length"
    );

    // Step 3: Create embedding point with payload
    let element_id = Uuid::new_v4();
    let payload = PointPayload::new("Blue Button", "button", "component", "tailwind")
        .with_confidence(0.85)
        .with_css_classes(vec![
            "bg-blue-500".to_string(),
            "rounded".to_string(),
            "hover:bg-blue-700".to_string(),
        ])
        .with_tags(vec!["interactive".to_string(), "primary".to_string()]);

    let point = EmbeddingPoint::new(element_id, embedding_result.embedding.clone(), payload);

    // Validate point dimensions
    assert!(
        point.validate(EMBEDDING_DIMENSIONS),
        "Point should have valid dimensions"
    );

    // Step 4: Store in Qdrant
    qdrant
        .upsert_point(TEST_COLLECTION, point)
        .await
        .expect("Failed to store point in Qdrant");

    // Step 5: Verify point exists via search
    let search_results = qdrant
        .search(
            TEST_COLLECTION,
            embedding_result.embedding.clone(),
            5,
            Some(SearchFilter::new()),
        )
        .await
        .expect("Search should succeed");

    // The point we just inserted should be in results (exact match = high score)
    let found = search_results.iter().any(|r| r.id == element_id);
    assert!(found, "Inserted point should be found in search results");

    // Verify the found result has correct payload
    let our_result = search_results.iter().find(|r| r.id == element_id).unwrap();
    assert_eq!(our_result.payload.name, "Blue Button");
    assert_eq!(our_result.payload.category, "button");
    assert_eq!(our_result.payload.design_system, "tailwind");
    assert_eq!(our_result.payload.confidence, 0.85);
    assert!(our_result.payload.css_classes.contains(&"bg-blue-500".to_string()));

    // Cleanup
    qdrant
        .delete_point(TEST_COLLECTION, element_id)
        .await
        .expect("Cleanup should succeed");
}

#[tokio::test]
#[ignore = "requires Qdrant connection"]
async fn test_batch_embedding_storage() {
    let qdrant = QdrantRepository::new(QdrantConfig::default())
        .await
        .expect("Failed to connect to Qdrant");

    qdrant.init_collections().await.ok();

    let embedding_generator = EmbeddingGenerator::new();

    // Generate multiple embeddings
    let test_elements = vec![
        ("Primary Button", "button", "A primary action button with blue background"),
        ("Card Container", "card", "A card with shadow and rounded corners"),
        ("Input Field", "input", "A text input with border and placeholder"),
    ];

    let mut points = Vec::new();
    let mut element_ids = Vec::new();

    for (name, category, description) in &test_elements {
        let embedding = embedding_generator
            .generate_text_embedding(description)
            .await
            .expect("Embedding should succeed");

        let id = Uuid::new_v4();
        element_ids.push(id);

        let payload = PointPayload::new(*name, *category, "component", "tailwind");
        let point = EmbeddingPoint::new(id, embedding.embedding, payload);
        points.push(point);
    }

    // Batch upsert
    let upserted = qdrant
        .upsert_batch(TEST_COLLECTION, points)
        .await
        .expect("Batch upsert should succeed");

    assert_eq!(upserted, 3, "Should upsert 3 points");

    // Verify collection info
    let info = qdrant
        .collection_info(TEST_COLLECTION)
        .await
        .expect("Should get collection info");

    assert!(
        info.points_count >= 3,
        "Collection should have at least 3 points"
    );

    // Cleanup
    qdrant
        .delete_batch(TEST_COLLECTION, element_ids)
        .await
        .expect("Cleanup should succeed");
}

#[tokio::test]
#[ignore = "requires Qdrant connection"]
async fn test_filtered_search() {
    let qdrant = QdrantRepository::new(QdrantConfig::default())
        .await
        .expect("Failed to connect to Qdrant");

    qdrant.init_collections().await.ok();

    let embedding_generator = EmbeddingGenerator::new();

    // Create points with different categories
    let mut element_ids = Vec::new();

    // Button
    let button_embedding = embedding_generator
        .generate_text_embedding("blue button component")
        .await
        .unwrap();
    let button_id = Uuid::new_v4();
    element_ids.push(button_id);
    let button_point = EmbeddingPoint::new(
        button_id,
        button_embedding.embedding.clone(),
        PointPayload::new("Test Button", "button", "component", "tailwind"),
    );

    // Card
    let card_embedding = embedding_generator
        .generate_text_embedding("card container component")
        .await
        .unwrap();
    let card_id = Uuid::new_v4();
    element_ids.push(card_id);
    let card_point = EmbeddingPoint::new(
        card_id,
        card_embedding.embedding,
        PointPayload::new("Test Card", "card", "component", "tailwind"),
    );

    // Store both
    qdrant.upsert_point(TEST_COLLECTION, button_point).await.ok();
    qdrant.upsert_point(TEST_COLLECTION, card_point).await.ok();

    // Search with filter for buttons only
    let filter = SearchFilter::new().with_category("button");
    let results = qdrant
        .search(TEST_COLLECTION, button_embedding.embedding, 10, Some(filter))
        .await
        .expect("Filtered search should succeed");

    // All results should be buttons
    for result in &results {
        assert_eq!(
            result.payload.category, "button",
            "Filter should only return buttons"
        );
    }

    // Cleanup
    qdrant.delete_batch(TEST_COLLECTION, element_ids).await.ok();
}

#[tokio::test]
#[ignore = "requires Qdrant connection"]
async fn test_similarity_search_ranking() {
    let qdrant = QdrantRepository::new(QdrantConfig::default())
        .await
        .expect("Failed to connect to Qdrant");

    qdrant.init_collections().await.ok();

    let embedding_generator = EmbeddingGenerator::new();

    // Create two similar and one different element
    let blue_button_text = "A blue primary button with hover effects";
    let red_button_text = "A red danger button with hover effects";
    let card_text = "A card component with image and text content";

    let blue_embedding = embedding_generator
        .generate_text_embedding(blue_button_text)
        .await
        .unwrap();
    let red_embedding = embedding_generator
        .generate_text_embedding(red_button_text)
        .await
        .unwrap();
    let card_embedding = embedding_generator
        .generate_text_embedding(card_text)
        .await
        .unwrap();

    let mut ids = Vec::new();

    let blue_id = Uuid::new_v4();
    ids.push(blue_id);
    qdrant
        .upsert_point(
            TEST_COLLECTION,
            EmbeddingPoint::new(
                blue_id,
                blue_embedding.embedding.clone(),
                PointPayload::new("Blue Button", "button", "component", "tailwind"),
            ),
        )
        .await
        .ok();

    let red_id = Uuid::new_v4();
    ids.push(red_id);
    qdrant
        .upsert_point(
            TEST_COLLECTION,
            EmbeddingPoint::new(
                red_id,
                red_embedding.embedding,
                PointPayload::new("Red Button", "button", "component", "tailwind"),
            ),
        )
        .await
        .ok();

    let card_id = Uuid::new_v4();
    ids.push(card_id);
    qdrant
        .upsert_point(
            TEST_COLLECTION,
            EmbeddingPoint::new(
                card_id,
                card_embedding.embedding,
                PointPayload::new("Card", "card", "component", "tailwind"),
            ),
        )
        .await
        .ok();

    // Search for "blue button" - should rank blue button highest
    let search_embedding = embedding_generator
        .generate_text_embedding("blue button with effects")
        .await
        .unwrap();

    let results = qdrant
        .search(
            TEST_COLLECTION,
            search_embedding.embedding,
            10,
            None,
        )
        .await
        .expect("Search should succeed");

    // Results should be ordered by similarity score (descending)
    for i in 1..results.len() {
        assert!(
            results[i - 1].score >= results[i].score,
            "Results should be ordered by score descending"
        );
    }

    // Cleanup
    qdrant.delete_batch(TEST_COLLECTION, ids).await.ok();
}

#[tokio::test]
async fn test_embedding_generator_fallback() {
    // This test runs without OPENAI_API_KEY to verify fallback works
    let generator = EmbeddingGenerator::new();

    let result = generator
        .generate_text_embedding("test text for embedding")
        .await
        .expect("Fallback embedding should work");

    // Fallback should still produce correct dimensions
    assert_eq!(
        result.dimensions, EMBEDDING_DIMENSIONS,
        "Fallback should produce correct dimensions"
    );
    assert_eq!(
        result.embedding.len(),
        EMBEDDING_DIMENSIONS,
        "Fallback vector should have correct length"
    );

    // Fallback should be deterministic for same input
    let result2 = generator
        .generate_text_embedding("test text for embedding")
        .await
        .unwrap();

    assert_eq!(
        result.embedding, result2.embedding,
        "Fallback should be deterministic"
    );
}

#[tokio::test]
async fn test_point_validation() {
    let valid_point = EmbeddingPoint::new(
        Uuid::new_v4(),
        vec![0.0; EMBEDDING_DIMENSIONS],
        PointPayload::new("Test", "button", "component", "tailwind"),
    );

    assert!(valid_point.validate(EMBEDDING_DIMENSIONS));

    let invalid_point = EmbeddingPoint::new(
        Uuid::new_v4(),
        vec![0.0; 512], // Wrong dimensions
        PointPayload::new("Test", "button", "component", "tailwind"),
    );

    assert!(!invalid_point.validate(EMBEDDING_DIMENSIONS));
}
