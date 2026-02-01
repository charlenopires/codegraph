//! MCP Server - stdio transport handler
//!
//! Implements JSON-RPC 2.0 over stdio for MCP protocol communication.

use std::env;
use std::io::{self, BufRead, Write};

use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::protocol::{
    error_codes, CallToolResult, InitializeResult, ListResourcesResult, ListToolsResult,
    Request, Response, ResourcesCapability, ServerCapabilities, ServerInfo,
    ToolsCapability,
};
use crate::resources;
use crate::tools;

/// Run the MCP server over stdio
pub async fn run_stdio() -> anyhow::Result<()> {
    let api_url = env::var("CODEGRAPH_API_URL").unwrap_or_else(|_| "http://localhost:3000".into());

    info!("Starting CodeGraph MCP server");
    info!("API URL: {}", api_url);
    info!("Protocol: JSON-RPC 2.0 over stdio");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to read stdin: {}", e);
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        debug!("Received: {}", line);

        let response = match serde_json::from_str::<Request>(&line) {
            Ok(request) => handle_request(request, &api_url).await,
            Err(e) => Response::error(None, error_codes::PARSE_ERROR, format!("Parse error: {}", e)),
        };

        let response_json = serde_json::to_string(&response)?;
        debug!("Sending: {}", response_json);

        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    info!("MCP server shutting down");
    Ok(())
}

async fn handle_request(request: Request, api_url: &str) -> Response {
    let id = request.id.clone();

    match request.method.as_str() {
        // Lifecycle methods
        "initialize" => handle_initialize(id),
        "initialized" => Response::success(id, json!({})),
        "ping" => Response::success(id, json!({})),

        // Tool methods
        "tools/list" => handle_list_tools(id),
        "tools/call" => handle_call_tool(id, request.params, api_url).await,

        // Resource methods
        "resources/list" => handle_list_resources(id),
        "resources/read" => handle_read_resource(id, request.params, api_url).await,

        // Unknown method
        _ => Response::error(
            id,
            error_codes::METHOD_NOT_FOUND,
            format!("Method not found: {}", request.method),
        ),
    }
}

fn handle_initialize(id: Option<Value>) -> Response {
    let result = InitializeResult {
        protocol_version: "2024-11-05",
        capabilities: ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: false }),
            resources: Some(ResourcesCapability {
                subscribe: false,
                list_changed: false,
            }),
        },
        server_info: ServerInfo {
            name: "codegraph-mcp",
            version: env!("CARGO_PKG_VERSION"),
        },
    };

    Response::success(id, serde_json::to_value(result).unwrap())
}

fn handle_list_tools(id: Option<Value>) -> Response {
    let result = ListToolsResult {
        tools: tools::list_tools(),
    };

    Response::success(id, serde_json::to_value(result).unwrap())
}

async fn handle_call_tool(id: Option<Value>, params: Option<Value>, api_url: &str) -> Response {
    let (name, arguments) = match params {
        Some(p) => {
            let name = p
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let arguments = p.get("arguments").cloned();
            (name, arguments)
        }
        None => {
            return Response::error(id, error_codes::INVALID_PARAMS, "Missing tool name");
        }
    };

    if name.is_empty() {
        return Response::error(id, error_codes::INVALID_PARAMS, "Tool name cannot be empty");
    }

    let result: CallToolResult = tools::call_tool(&name, arguments, api_url).await;
    Response::success(id, serde_json::to_value(result).unwrap())
}

fn handle_list_resources(id: Option<Value>) -> Response {
    let result = ListResourcesResult {
        resources: resources::list_resources(),
    };

    Response::success(id, serde_json::to_value(result).unwrap())
}

async fn handle_read_resource(id: Option<Value>, params: Option<Value>, api_url: &str) -> Response {
    let uri = match params {
        Some(p) => p
            .get("uri")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        None => {
            return Response::error(id, error_codes::INVALID_PARAMS, "Missing resource URI");
        }
    };

    if uri.is_empty() {
        return Response::error(id, error_codes::INVALID_PARAMS, "Resource URI cannot be empty");
    }

    match resources::read_resource(&uri, api_url).await {
        Ok(result) => Response::success(id, serde_json::to_value(result).unwrap()),
        Err(e) => Response::error(id, error_codes::RESOURCE_NOT_FOUND, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_initialize() {
        let response = handle_initialize(Some(json!(1)));
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert_eq!(result["serverInfo"]["name"], "codegraph-mcp");
        assert!(result["capabilities"]["tools"].is_object());
        assert!(result["capabilities"]["resources"].is_object());
    }

    #[test]
    fn test_handle_list_tools() {
        let response = handle_list_tools(Some(json!(1)));
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn test_handle_list_resources() {
        let response = handle_list_resources(Some(json!(1)));
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        let resources = result["resources"].as_array().unwrap();
        assert_eq!(resources.len(), 2);
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let request = Request {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "unknown/method".to_string(),
            params: None,
        };

        let response = handle_request(request, "http://localhost:3000").await;
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, error_codes::METHOD_NOT_FOUND);
    }

    #[tokio::test]
    async fn test_handle_call_tool_missing_name() {
        let response = handle_call_tool(Some(json!(1)), None, "http://localhost:3000").await;
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, error_codes::INVALID_PARAMS);
    }
}
