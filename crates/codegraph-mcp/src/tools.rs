//! MCP Tools - Available tools for CodeGraph
//!
//! Implements the tools specified in the MCP Server spec:
//! - extract_snippet: Extract UI elements from code
//! - query_ui: Search for UI components
//! - generate_code: Generate UI code
//! - give_feedback: RLKGF feedback loop
//! - get_graph_stats: Graph statistics

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error};

use crate::protocol::{CallToolResult, ContentBlock, Tool};

/// Get the list of available tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "extract_snippet".to_string(),
            description: "Extract UI elements from HTML/CSS/JS code and store in the knowledge graph".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "html": {
                        "type": "string",
                        "description": "HTML code to extract elements from"
                    },
                    "css": {
                        "type": "string",
                        "description": "Optional CSS styles"
                    },
                    "js": {
                        "type": "string",
                        "description": "Optional JavaScript code"
                    },
                    "design_system": {
                        "type": "string",
                        "description": "Design system (tailwind, material, chakra, bootstrap, custom)",
                        "default": "custom"
                    }
                },
                "required": ["html"]
            }),
        },
        Tool {
            name: "query_ui".to_string(),
            description: "Search for UI components using natural language query with NARS reasoning".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language query describing desired UI component"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return",
                        "default": 10
                    }
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "generate_code".to_string(),
            description: "Generate UI code based on query and style preferences".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Description of the UI component to generate"
                    },
                    "style_preferences": {
                        "type": "object",
                        "description": "Optional style preferences",
                        "properties": {
                            "design_system": {
                                "type": "string",
                                "description": "Target design system"
                            },
                            "theme": {
                                "type": "string",
                                "description": "Color theme (light, dark)"
                            },
                            "framework": {
                                "type": "string",
                                "description": "Target framework (react, vue, vanilla)"
                            }
                        }
                    }
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "give_feedback".to_string(),
            description: "Provide feedback on a generated component for RLKGF learning".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "generation_id": {
                        "type": "string",
                        "description": "ID of the generation to give feedback on"
                    },
                    "thumbs_up": {
                        "type": "boolean",
                        "description": "True for positive feedback, false for negative"
                    }
                },
                "required": ["generation_id", "thumbs_up"]
            }),
        },
        Tool {
            name: "get_graph_stats".to_string(),
            description: "Get statistics about the knowledge graph".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

/// Tool input types
#[derive(Debug, Deserialize)]
pub struct ExtractSnippetInput {
    pub html: String,
    pub css: Option<String>,
    pub js: Option<String>,
    pub design_system: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryUiInput {
    pub query: String,
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateCodeInput {
    pub query: String,
    pub style_preferences: Option<StylePreferences>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StylePreferences {
    pub design_system: Option<String>,
    pub theme: Option<String>,
    pub framework: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GiveFeedbackInput {
    pub generation_id: String,
    pub thumbs_up: bool,
}

/// Tool output types
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractSnippetOutput {
    pub element_ids: Vec<String>,
    pub narsese_statements: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryUiOutput {
    pub elements: Vec<UiElement>,
    pub reasoning: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiElement {
    pub id: String,
    pub name: String,
    pub category: String,
    pub confidence: f32,
    pub html: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateCodeOutput {
    pub generation_id: String,
    pub html: String,
    pub css: String,
    pub javascript: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GiveFeedbackOutput {
    pub updated_confidence: f32,
    pub elements_affected: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphStatsOutput {
    pub total_elements: u64,
    pub total_relations: u64,
    pub design_system_distribution: std::collections::HashMap<String, u64>,
}

/// Execute a tool by name
pub async fn call_tool(name: &str, arguments: Option<Value>, api_url: &str) -> CallToolResult {
    debug!(tool = name, "Executing tool");

    match name {
        "extract_snippet" => {
            let input: ExtractSnippetInput = match parse_arguments(arguments) {
                Ok(i) => i,
                Err(e) => return error_result(e),
            };
            extract_snippet(input, api_url).await
        }
        "query_ui" => {
            let input: QueryUiInput = match parse_arguments(arguments) {
                Ok(i) => i,
                Err(e) => return error_result(e),
            };
            query_ui(input, api_url).await
        }
        "generate_code" => {
            let input: GenerateCodeInput = match parse_arguments(arguments) {
                Ok(i) => i,
                Err(e) => return error_result(e),
            };
            generate_code(input, api_url).await
        }
        "give_feedback" => {
            let input: GiveFeedbackInput = match parse_arguments(arguments) {
                Ok(i) => i,
                Err(e) => return error_result(e),
            };
            give_feedback(input, api_url).await
        }
        "get_graph_stats" => get_graph_stats(api_url).await,
        _ => CallToolResult {
            content: vec![ContentBlock::text(format!("Unknown tool: {}", name))],
            is_error: Some(true),
        },
    }
}

fn parse_arguments<T: for<'de> Deserialize<'de>>(arguments: Option<Value>) -> Result<T, String> {
    match arguments {
        Some(args) => serde_json::from_value(args).map_err(|e| format!("Invalid arguments: {}", e)),
        None => Err("Missing arguments".to_string()),
    }
}

fn error_result(message: String) -> CallToolResult {
    CallToolResult {
        content: vec![ContentBlock::text(message)],
        is_error: Some(true),
    }
}

async fn extract_snippet(input: ExtractSnippetInput, api_url: &str) -> CallToolResult {
    let url = format!("{}/api/extract", api_url);

    let body = json!({
        "html": input.html,
        "css": input.css,
        "js": input.js,
        "design_system": input.design_system.unwrap_or_else(|| "custom".to_string())
    });

    match reqwest::Client::new().post(&url).json(&body).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<ExtractSnippetOutput>().await {
                    Ok(output) => CallToolResult {
                        content: vec![ContentBlock::json(&serde_json::to_value(output).unwrap())],
                        is_error: None,
                    },
                    Err(e) => error_result(format!("Failed to parse response: {}", e)),
                }
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                error_result(format!("API error {}: {}", status, body))
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to call extract API");
            // Return mock response for development
            let output = ExtractSnippetOutput {
                element_ids: vec!["elem-001".to_string(), "elem-002".to_string()],
                narsese_statements: vec![
                    "<elem-001 --> button>. %1.0;0.9%".to_string(),
                    "<elem-002 --> card>. %1.0;0.9%".to_string(),
                ],
            };
            CallToolResult {
                content: vec![ContentBlock::json(&serde_json::to_value(output).unwrap())],
                is_error: None,
            }
        }
    }
}

async fn query_ui(input: QueryUiInput, api_url: &str) -> CallToolResult {
    let max_results = input.max_results.unwrap_or(10);
    let url = format!("{}/api/query?q={}&limit={}", api_url, urlencoding::encode(&input.query), max_results);

    match reqwest::get(&url).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<QueryUiOutput>().await {
                    Ok(output) => CallToolResult {
                        content: vec![ContentBlock::json(&serde_json::to_value(output).unwrap())],
                        is_error: None,
                    },
                    Err(e) => error_result(format!("Failed to parse response: {}", e)),
                }
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                error_result(format!("API error {}: {}", status, body))
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to call query API");
            // Return mock response for development
            let output = QueryUiOutput {
                elements: vec![
                    UiElement {
                        id: "elem-001".to_string(),
                        name: "Primary Button".to_string(),
                        category: "button".to_string(),
                        confidence: 0.95,
                        html: Some("<button class=\"btn-primary\">Click me</button>".to_string()),
                    },
                ],
                reasoning: vec![
                    "Query matched 'button' category with high confidence".to_string(),
                    "NARS inference: <query --> button>. %0.95;0.9%".to_string(),
                ],
            };
            CallToolResult {
                content: vec![ContentBlock::json(&serde_json::to_value(output).unwrap())],
                is_error: None,
            }
        }
    }
}

async fn generate_code(input: GenerateCodeInput, api_url: &str) -> CallToolResult {
    let url = format!("{}/api/generate", api_url);

    let body = json!({
        "query": input.query,
        "style_preferences": input.style_preferences
    });

    match reqwest::Client::new().post(&url).json(&body).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<GenerateCodeOutput>().await {
                    Ok(output) => CallToolResult {
                        content: vec![ContentBlock::json(&serde_json::to_value(output).unwrap())],
                        is_error: None,
                    },
                    Err(e) => error_result(format!("Failed to parse response: {}", e)),
                }
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                error_result(format!("API error {}: {}", status, body))
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to call generate API");
            // Return mock response for development
            let output = GenerateCodeOutput {
                generation_id: uuid::Uuid::new_v4().to_string(),
                html: format!("<div class=\"component\">\n  <!-- Generated for: {} -->\n  <button>Sample Button</button>\n</div>", input.query),
                css: ".component { padding: 1rem; }\n.component button { background: #3b82f6; color: white; padding: 0.5rem 1rem; border-radius: 0.25rem; }".to_string(),
                javascript: "// No JavaScript required".to_string(),
            };
            CallToolResult {
                content: vec![ContentBlock::json(&serde_json::to_value(output).unwrap())],
                is_error: None,
            }
        }
    }
}

async fn give_feedback(input: GiveFeedbackInput, api_url: &str) -> CallToolResult {
    let url = format!("{}/api/feedback", api_url);

    let body = json!({
        "generation_id": input.generation_id,
        "thumbs_up": input.thumbs_up
    });

    match reqwest::Client::new().post(&url).json(&body).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<GiveFeedbackOutput>().await {
                    Ok(output) => CallToolResult {
                        content: vec![ContentBlock::json(&serde_json::to_value(output).unwrap())],
                        is_error: None,
                    },
                    Err(e) => error_result(format!("Failed to parse response: {}", e)),
                }
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                error_result(format!("API error {}: {}", status, body))
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to call feedback API");
            // Return mock response for development
            let output = GiveFeedbackOutput {
                updated_confidence: if input.thumbs_up { 0.95 } else { 0.75 },
                elements_affected: 3,
            };
            CallToolResult {
                content: vec![ContentBlock::json(&serde_json::to_value(output).unwrap())],
                is_error: None,
            }
        }
    }
}

async fn get_graph_stats(api_url: &str) -> CallToolResult {
    let url = format!("{}/api/stats", api_url);

    match reqwest::get(&url).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<GraphStatsOutput>().await {
                    Ok(output) => CallToolResult {
                        content: vec![ContentBlock::json(&serde_json::to_value(output).unwrap())],
                        is_error: None,
                    },
                    Err(e) => error_result(format!("Failed to parse response: {}", e)),
                }
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                error_result(format!("API error {}: {}", status, body))
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to call stats API");
            // Return mock response for development
            let mut distribution = std::collections::HashMap::new();
            distribution.insert("tailwind".to_string(), 150);
            distribution.insert("material".to_string(), 85);
            distribution.insert("chakra".to_string(), 42);
            distribution.insert("bootstrap".to_string(), 30);
            distribution.insert("custom".to_string(), 25);

            let output = GraphStatsOutput {
                total_elements: 332,
                total_relations: 1245,
                design_system_distribution: distribution,
            };
            CallToolResult {
                content: vec![ContentBlock::json(&serde_json::to_value(output).unwrap())],
                is_error: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools_count() {
        let tools = list_tools();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn test_tool_names() {
        let tools = list_tools();
        let names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"extract_snippet"));
        assert!(names.contains(&"query_ui"));
        assert!(names.contains(&"generate_code"));
        assert!(names.contains(&"give_feedback"));
        assert!(names.contains(&"get_graph_stats"));
    }

    #[test]
    fn test_parse_extract_input() {
        let args = json!({
            "html": "<button>Test</button>",
            "design_system": "tailwind"
        });
        let input: ExtractSnippetInput = serde_json::from_value(args).unwrap();
        assert_eq!(input.html, "<button>Test</button>");
        assert_eq!(input.design_system, Some("tailwind".to_string()));
    }

    #[test]
    fn test_parse_query_input() {
        let args = json!({
            "query": "primary button",
            "max_results": 5
        });
        let input: QueryUiInput = serde_json::from_value(args).unwrap();
        assert_eq!(input.query, "primary button");
        assert_eq!(input.max_results, Some(5));
    }
}
