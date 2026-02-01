//! Integration Test: Generate → LLM → Valid code output
//!
//! This test validates code generation produces valid output.
//!
//! Prerequisites:
//! - OPENAI_API_KEY environment variable (for real LLM tests)
//! - Tests with fallback work without API key
//!
//! Run with: cargo test --test test_code_generation

use codegraph_generation::{
    generator::GenerationRequest,
    prompt::SimilarElement,
    CodeParser, VanillaCodeGenerator,
};

/// Create a simple generation request
fn create_test_request(description: &str) -> GenerationRequest {
    GenerationRequest {
        description: description.to_string(),
        similar_elements: vec![
            SimilarElement {
                name: "Primary Button".to_string(),
                category: "button".to_string(),
                tags: vec!["interactive".to_string(), "primary".to_string()],
                similarity: 0.85,
            },
        ],
        reasoning: Some("User needs a clickable button element".to_string()),
        categories: vec!["button".to_string()],
    }
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_generate_button_component() {
    let generator = VanillaCodeGenerator::new();

    let request = create_test_request("Create a blue primary button with hover effect");

    let result = generator
        .generate(request)
        .await
        .expect("Generation should succeed");

    // Verify response contains valid HTML
    assert!(
        result.code.html.is_some(),
        "Should generate HTML code"
    );

    let html = result.code.html.as_ref().unwrap();
    assert!(
        html.contains("<button") || html.contains("<Button"),
        "Should contain a button element"
    );

    // Verify HTML document is valid
    assert!(
        result.html_document.contains("<!DOCTYPE html>"),
        "Should generate valid HTML document"
    );
    assert!(
        result.html_document.contains("<html"),
        "Should have html tag"
    );

    // Validation errors should be empty or minimal
    if !result.validation_errors.is_empty() {
        println!("Validation warnings: {:?}", result.validation_errors);
    }
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_generate_card_component() {
    let generator = VanillaCodeGenerator::new();

    let request = GenerationRequest {
        description: "Create a card component with image, title, and description".to_string(),
        similar_elements: vec![
            SimilarElement {
                name: "Product Card".to_string(),
                category: "card".to_string(),
                tags: vec!["container".to_string(), "shadow".to_string()],
                similarity: 0.9,
            },
        ],
        reasoning: Some("Display product information in a card layout".to_string()),
        categories: vec!["card".to_string(), "container".to_string()],
    };

    let result = generator
        .generate(request)
        .await
        .expect("Generation should succeed");

    assert!(result.code.html.is_some(), "Should generate HTML");

    let html = result.code.html.as_ref().unwrap();
    assert!(
        html.contains("<div") || html.contains("class="),
        "Should contain div elements or classes"
    );
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_generate_with_css() {
    let generator = VanillaCodeGenerator::new();

    let request = GenerationRequest {
        description: "Create a styled navigation bar with flexbox layout".to_string(),
        similar_elements: vec![],
        reasoning: None,
        categories: vec!["navigation".to_string(), "layout".to_string()],
    };

    let result = generator
        .generate(request)
        .await
        .expect("Generation should succeed");

    // Should have HTML
    assert!(result.code.html.is_some(), "Should generate HTML");

    // May also have CSS
    if let Some(css) = &result.code.css {
        assert!(!css.is_empty(), "CSS should not be empty if present");
        // Check for common CSS patterns
        let has_css_patterns = css.contains("{")
            && css.contains("}")
            && (css.contains(":") || css.contains("flex") || css.contains("display"));
        assert!(has_css_patterns, "Should contain valid CSS syntax");
    }

    // Full document should compile everything
    assert!(
        result.html_document.contains("<style>") || result.code.css.is_none(),
        "Document should include styles if CSS was generated"
    );
}

#[tokio::test]
async fn test_code_parser_extracts_html() {
    let parser = CodeParser::new();

    let llm_response = r#"
Here's a button component:

```html
<button class="btn-primary">Click me</button>
```

This button uses Tailwind CSS classes.
"#;

    let code = parser.parse(llm_response);

    assert!(code.html.is_some(), "Should extract HTML block");
    let html = code.html.unwrap();
    assert!(html.contains("<button"), "Should contain button tag");
    assert!(html.contains("btn-primary"), "Should preserve classes");
}

#[tokio::test]
async fn test_code_parser_extracts_css() {
    let parser = CodeParser::new();

    let llm_response = r#"
```html
<div class="card">Content</div>
```

```css
.card {
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}
```
"#;

    let code = parser.parse(llm_response);

    assert!(code.html.is_some(), "Should extract HTML");
    assert!(code.css.is_some(), "Should extract CSS");

    let css = code.css.unwrap();
    assert!(css.contains(".card"), "Should contain class selector");
    assert!(css.contains("border-radius"), "Should contain CSS properties");
}

#[tokio::test]
async fn test_code_parser_extracts_javascript() {
    let parser = CodeParser::new();

    let llm_response = r#"
```html
<button id="btn">Click</button>
```

```javascript
document.getElementById('btn').addEventListener('click', () => {
    alert('Clicked!');
});
```
"#;

    let code = parser.parse(llm_response);

    assert!(code.html.is_some(), "Should extract HTML");
    assert!(code.javascript.is_some(), "Should extract JavaScript");

    let js = code.javascript.unwrap();
    assert!(js.contains("addEventListener"), "Should contain JS code");
}

#[tokio::test]
async fn test_parsed_code_to_html_document() {
    let parser = CodeParser::new();

    let llm_response = r#"
```html
<button class="btn">Submit</button>
```

```css
.btn { background: blue; color: white; }
```

```javascript
console.log('loaded');
```
"#;

    let code = parser.parse(llm_response);
    let document = code.to_html_document();

    // Verify document structure
    assert!(document.contains("<!DOCTYPE html>"), "Should have doctype");
    assert!(document.contains("<html"), "Should have html tag");
    assert!(document.contains("<head>"), "Should have head");
    assert!(document.contains("<body>"), "Should have body");

    // Verify content is embedded
    assert!(document.contains("<style>"), "Should embed CSS in style tag");
    assert!(document.contains("<script>"), "Should embed JS in script tag");
    assert!(document.contains("btn"), "Should include button class");
}

#[tokio::test]
async fn test_code_parser_handles_empty_response() {
    let parser = CodeParser::new();

    let code = parser.parse("No code blocks here, just text.");

    assert!(code.is_empty(), "Should detect empty code");
    assert!(code.html.is_none());
    assert!(code.css.is_none());
    assert!(code.javascript.is_none());
}

#[tokio::test]
async fn test_code_parser_handles_malformed_blocks() {
    let parser = CodeParser::new();

    // Missing closing backticks
    let response = "```html\n<div>Incomplete";
    let code = parser.parse(response);

    // Should handle gracefully (may or may not extract)
    // The important thing is it doesn't panic
    let _ = code.to_html_document();
}

#[tokio::test]
async fn test_generation_request_creation() {
    let request = GenerationRequest {
        description: "A simple button".to_string(),
        similar_elements: vec![
            SimilarElement {
                name: "Button".to_string(),
                category: "button".to_string(),
                tags: vec!["primary".to_string()],
                similarity: 0.9,
            },
        ],
        reasoning: Some("User wants a button".to_string()),
        categories: vec!["button".to_string()],
    };

    assert_eq!(request.description, "A simple button");
    assert_eq!(request.similar_elements.len(), 1);
    assert_eq!(request.similar_elements[0].similarity, 0.9);
}

#[tokio::test]
async fn test_html_validation() {
    let parser = CodeParser::new();

    // Valid HTML
    let valid_html = "<div><p>Hello</p></div>";
    let result = parser.validate_html(valid_html);
    assert!(result.is_ok(), "Valid HTML should pass validation");

    // Note: The parser may or may not catch all HTML errors
    // depending on implementation. The key is it doesn't panic.
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_generate_with_context_from_similar_elements() {
    let generator = VanillaCodeGenerator::new();

    // Provide rich context from similar elements
    let request = GenerationRequest {
        description: "Create a form input with validation".to_string(),
        similar_elements: vec![
            SimilarElement {
                name: "Email Input".to_string(),
                category: "input".to_string(),
                tags: vec!["form".to_string(), "validation".to_string()],
                similarity: 0.92,
            },
            SimilarElement {
                name: "Text Field".to_string(),
                category: "input".to_string(),
                tags: vec!["form".to_string(), "text".to_string()],
                similarity: 0.88,
            },
        ],
        reasoning: Some("Form needs input validation for email addresses".to_string()),
        categories: vec!["input".to_string(), "form".to_string()],
    };

    let result = generator
        .generate(request)
        .await
        .expect("Generation should succeed");

    assert!(result.code.html.is_some());

    let html = result.code.html.as_ref().unwrap();
    assert!(
        html.contains("<input") || html.contains("type="),
        "Should generate input element"
    );
}
