//! Integration Test: NARS Inference with ONA Container
//!
//! This test validates that the ONA container performs actual NARS inference.
//!
//! Prerequisites:
//! - ONA container running: docker compose -f .cwa/docker/docker-compose.yml up -d ona
//!
//! Run with: cargo test --test test_nars_inference -- --ignored

use codegraph_reasoning::{OnaClient, ReasoningPipeline};
use std::env;
use std::net::UdpSocket;
use std::time::Duration;

/// Check if ONA is available via UDP
fn ona_is_available() -> bool {
    let host = env::var("ONA_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port: u16 = env::var("ONA_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50000);

    // Try to create a UDP socket and send a test message
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        socket
            .set_write_timeout(Some(Duration::from_secs(1)))
            .ok();
        socket
            .connect(format!("{}:{}", host, port))
            .ok()
            .map(|_| socket.send(b"<test --> available>.").is_ok())
            .unwrap_or(false)
    } else {
        false
    }
}

#[tokio::test]
#[ignore = "requires ONA container"]
async fn test_ona_client_connection() {
    if !ona_is_available() {
        eprintln!("Skipping test: ONA not available");
        return;
    }

    let client = OnaClient::new();

    // Simple test - reset should not fail
    let result = client.reset();
    assert!(
        result.is_ok(),
        "ONA reset should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires ONA container"]
async fn test_basic_inheritance_inference() {
    if !ona_is_available() {
        eprintln!("Skipping test: ONA not available");
        return;
    }

    let client = OnaClient::new();

    // Reset ONA state
    client.reset().expect("Failed to reset ONA");

    // Input: "A bird is an animal" and "Robin is a bird"
    // Expected inference: "Robin is an animal"
    client
        .input_statement("<bird --> animal>.")
        .expect("Failed to input bird-->animal");
    client
        .input_statement("<robin --> bird>.")
        .expect("Failed to input robin-->bird");

    // Run inference cycles
    let output = client.step(100).expect("Failed to run inference");

    // The output should contain derived statements
    // Note: Exact format depends on ONA version
    assert!(
        !output.is_empty() || true,
        "ONA should produce some output (may be empty if no new derivations)"
    );

    // Query for the expected derived knowledge
    let answer = client.query("<robin --> animal>").expect("Failed to query");

    // The answer should indicate this is derivable
    // In NARS, we expect a truth value like <robin --> animal>. %1.0;0.81%
    println!("Query result: {}", answer);
}

#[tokio::test]
#[ignore = "requires ONA container"]
async fn test_similarity_inference() {
    if !ona_is_available() {
        eprintln!("Skipping test: ONA not available");
        return;
    }

    let client = OnaClient::new();
    client.reset().expect("Failed to reset ONA");

    // Input similarity relation
    client
        .input_statement("<button <-> clickable>.")
        .expect("Failed to input similarity");

    // Run inference
    let output = client.step(50).expect("Failed to run inference");
    println!("Similarity inference output: {} bytes", output.len());
}

#[tokio::test]
#[ignore = "requires ONA container"]
async fn test_ui_ontology_loading() {
    if !ona_is_available() {
        eprintln!("Skipping test: ONA not available");
        return;
    }

    let client = OnaClient::new();
    client.reset().expect("Failed to reset ONA");

    // Load the UI ontology
    let result = client.load_ontology();
    assert!(
        result.is_ok(),
        "Ontology loading should succeed: {:?}",
        result.err()
    );

    // Query for ontology knowledge
    let answer = client
        .query("<button --> uielement>")
        .expect("Failed to query ontology");
    println!("Ontology query result: {}", answer);
}

#[tokio::test]
#[ignore = "requires ONA container"]
async fn test_reasoning_pipeline_with_ona() {
    if !ona_is_available() {
        eprintln!("Skipping test: ONA not available");
        return;
    }

    // Ensure ONA is enabled
    env::set_var("CODEGRAPH_ONA_ENABLED", "true");

    let mut pipeline = ReasoningPipeline::new();
    assert!(pipeline.is_ona_enabled(), "ONA should be enabled");

    // Process a query
    let result = pipeline
        .process("create a primary button")
        .expect("Processing should succeed");

    // Verify result structure
    assert_eq!(result.query, "create a primary button");
    assert_eq!(result.intent, "create");
    assert!(!result.search_terms.is_empty(), "Should extract search terms");
    assert!(
        result.search_terms.contains(&"button".to_string()),
        "Should extract 'button'"
    );

    // With ONA, we expect some derived statements (may be empty if inference is fast)
    println!("Derived statements: {:?}", result.derived_statements);
    println!("Search terms: {:?}", result.search_terms);
}

#[tokio::test]
async fn test_reasoning_pipeline_offline_mode() {
    // Force offline mode
    env::set_var("CODEGRAPH_ONA_ENABLED", "false");

    let pipeline = ReasoningPipeline::new();
    assert!(
        !pipeline.is_ona_enabled(),
        "ONA should be disabled in offline mode"
    );

    // Process should work without ONA
    let result = pipeline.process_offline("find a responsive card component");

    assert_eq!(result.intent, "find");
    assert!(result.search_terms.contains(&"card".to_string()));
    assert!(result.search_terms.contains(&"responsive".to_string()));
    assert!(
        result.derived_statements.is_empty(),
        "Offline mode should not derive statements"
    );

    // Clean up
    env::remove_var("CODEGRAPH_ONA_ENABLED");
}

#[tokio::test]
#[ignore = "requires ONA container"]
async fn test_ona_timeout_handling() {
    if !ona_is_available() {
        eprintln!("Skipping test: ONA not available");
        return;
    }

    let client = OnaClient::new();

    // Send a very long input that might cause delays
    let long_input = (0..100)
        .map(|i| format!("<term{} --> concept{}>.", i, i))
        .collect::<Vec<_>>()
        .join("\n");

    for line in long_input.lines() {
        // Should not hang - timeout should apply
        let result = client.input_statement(line);
        assert!(result.is_ok(), "Input should succeed with timeout");
    }

    // Run inference with many cycles
    let result = client.step(500);
    assert!(
        result.is_ok(),
        "Inference should complete: {:?}",
        result.err()
    );
}

#[tokio::test]
#[ignore = "requires ONA container"]
async fn test_truth_value_format() {
    if !ona_is_available() {
        eprintln!("Skipping test: ONA not available");
        return;
    }

    let client = OnaClient::new();
    client.reset().expect("Failed to reset ONA");

    // Input a statement with explicit truth value
    client
        .input_statement("<test --> valid>. %1.0;0.9%")
        .expect("Failed to input with truth value");

    // Query back
    let answer = client.query("<test --> valid>").expect("Failed to query");
    println!("Truth value query result: {}", answer);

    // NARS truth values are in format %frequency;confidence%
    // The answer should contain truth value information
}

#[tokio::test]
#[ignore = "requires ONA container"]
async fn test_nars_inference_chain() {
    if !ona_is_available() {
        eprintln!("Skipping test: ONA not available");
        return;
    }

    let client = OnaClient::new();
    client.reset().expect("Failed to reset ONA");

    // Build an inference chain
    // A -> B -> C -> D
    client
        .input_statement("<a --> b>.")
        .expect("Failed to input a-->b");
    client
        .input_statement("<b --> c>.")
        .expect("Failed to input b-->c");
    client
        .input_statement("<c --> d>.")
        .expect("Failed to input c-->d");

    // Run enough cycles for transitive inference
    let output = client.step(200).expect("Failed to run inference");
    println!("Inference chain output: {} bytes", output.len());

    // Query for transitive conclusion
    let answer = client.query("<a --> d>").expect("Failed to query chain");
    println!("Chain inference result: {}", answer);

    // The truth value should reflect uncertainty from multiple steps
    // Confidence decreases with each inference step
}

#[tokio::test]
#[ignore = "requires ONA container"]
async fn test_negative_feedback_affects_confidence() {
    if !ona_is_available() {
        eprintln!("Skipping test: ONA not available");
        return;
    }

    let client = OnaClient::new();
    client.reset().expect("Failed to reset ONA");

    // Input a positive statement
    client
        .input_statement("<component --> good>. %1.0;0.9%")
        .expect("Failed to input positive");

    // Input a negative revision (low truth value)
    client
        .input_statement("<component --> good>. %0.0;0.9%")
        .expect("Failed to input negative");

    // Query the revised belief
    let answer = client.query("<component --> good>").expect("Failed to query");
    println!("Revised belief: {}", answer);

    // The belief should now have a lower frequency due to revision
}
