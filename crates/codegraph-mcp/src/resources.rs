//! MCP Resources - Available resources for CodeGraph
//!
//! Implements the resources specified in the MCP Server spec:
//! - codegraph://metrics - RLKGF metrics
//! - codegraph://recent - Recent generations

use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::protocol::{ReadResourceResult, Resource, ResourceContent};

/// Get the list of available resources
pub fn list_resources() -> Vec<Resource> {
    vec![
        Resource {
            uri: "codegraph://metrics".to_string(),
            name: "RLKGF Metrics".to_string(),
            description: Some("Current RLKGF feedback loop metrics including confidence scores and feedback counts".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        Resource {
            uri: "codegraph://recent".to_string(),
            name: "Recent Generations".to_string(),
            description: Some("List of recent code generations with their metadata and feedback status".to_string()),
            mime_type: Some("application/json".to_string()),
        },
    ]
}

/// RLKGF Metrics data
#[derive(Debug, Serialize, Deserialize)]
pub struct RlkgfMetrics {
    pub total_feedback: u64,
    pub positive_feedback: u64,
    pub negative_feedback: u64,
    pub positive_ratio: f64,
    pub average_confidence: f64,
    pub confidence_delta_sum: f64,
    pub elements_updated: u64,
}

/// Recent generation entry
#[derive(Debug, Serialize, Deserialize)]
pub struct RecentGeneration {
    pub id: String,
    pub query: String,
    pub created_at: String,
    pub feedback: Option<bool>,
    pub confidence: f64,
    pub elements_used: Vec<String>,
}

/// Recent generations list
#[derive(Debug, Serialize, Deserialize)]
pub struct RecentGenerations {
    pub generations: Vec<RecentGeneration>,
    pub total_count: u64,
}

/// Read a resource by URI
pub async fn read_resource(uri: &str, api_url: &str) -> Result<ReadResourceResult, String> {
    debug!(uri = uri, "Reading resource");

    match uri {
        "codegraph://metrics" => read_metrics_resource(api_url).await,
        "codegraph://recent" => read_recent_resource(api_url).await,
        _ => Err(format!("Resource not found: {}", uri)),
    }
}

async fn read_metrics_resource(api_url: &str) -> Result<ReadResourceResult, String> {
    let url = format!("{}/api/metrics/rlkgf", api_url);

    match reqwest::get(&url).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<RlkgfMetrics>().await {
                    Ok(metrics) => Ok(ReadResourceResult {
                        contents: vec![ResourceContent::json_content(
                            "codegraph://metrics",
                            &serde_json::to_value(metrics).unwrap(),
                        )],
                    }),
                    Err(e) => Err(format!("Failed to parse metrics: {}", e)),
                }
            } else {
                Err(format!("API error: {}", resp.status()))
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch metrics from API");
            // Return mock data for development
            let metrics = RlkgfMetrics {
                total_feedback: 156,
                positive_feedback: 128,
                negative_feedback: 28,
                positive_ratio: 0.82,
                average_confidence: 0.87,
                confidence_delta_sum: 12.5,
                elements_updated: 423,
            };
            Ok(ReadResourceResult {
                contents: vec![ResourceContent::json_content(
                    "codegraph://metrics",
                    &serde_json::to_value(metrics).unwrap(),
                )],
            })
        }
    }
}

async fn read_recent_resource(api_url: &str) -> Result<ReadResourceResult, String> {
    let url = format!("{}/api/generations/recent", api_url);

    match reqwest::get(&url).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<RecentGenerations>().await {
                    Ok(generations) => Ok(ReadResourceResult {
                        contents: vec![ResourceContent::json_content(
                            "codegraph://recent",
                            &serde_json::to_value(generations).unwrap(),
                        )],
                    }),
                    Err(e) => Err(format!("Failed to parse generations: {}", e)),
                }
            } else {
                Err(format!("API error: {}", resp.status()))
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch recent generations from API");
            // Return mock data for development
            let generations = RecentGenerations {
                generations: vec![
                    RecentGeneration {
                        id: "gen-001".to_string(),
                        query: "primary button with loading state".to_string(),
                        created_at: "2024-01-15T10:30:00Z".to_string(),
                        feedback: Some(true),
                        confidence: 0.92,
                        elements_used: vec!["elem-001".to_string(), "elem-002".to_string()],
                    },
                    RecentGeneration {
                        id: "gen-002".to_string(),
                        query: "card with image and description".to_string(),
                        created_at: "2024-01-15T10:25:00Z".to_string(),
                        feedback: None,
                        confidence: 0.85,
                        elements_used: vec!["elem-003".to_string()],
                    },
                    RecentGeneration {
                        id: "gen-003".to_string(),
                        query: "navigation sidebar".to_string(),
                        created_at: "2024-01-15T10:20:00Z".to_string(),
                        feedback: Some(false),
                        confidence: 0.78,
                        elements_used: vec!["elem-004".to_string(), "elem-005".to_string()],
                    },
                ],
                total_count: 156,
            };
            Ok(ReadResourceResult {
                contents: vec![ResourceContent::json_content(
                    "codegraph://recent",
                    &serde_json::to_value(generations).unwrap(),
                )],
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_resources() {
        let resources = list_resources();
        assert_eq!(resources.len(), 2);

        let uris: Vec<_> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&"codegraph://metrics"));
        assert!(uris.contains(&"codegraph://recent"));
    }

    #[test]
    fn test_resource_mime_types() {
        let resources = list_resources();
        for resource in resources {
            assert_eq!(resource.mime_type, Some("application/json".to_string()));
        }
    }

    #[test]
    fn test_metrics_serialization() {
        let metrics = RlkgfMetrics {
            total_feedback: 100,
            positive_feedback: 80,
            negative_feedback: 20,
            positive_ratio: 0.8,
            average_confidence: 0.85,
            confidence_delta_sum: 5.5,
            elements_updated: 200,
        };

        let json = serde_json::to_value(&metrics).unwrap();
        assert_eq!(json["total_feedback"], 100);
        assert_eq!(json["positive_ratio"], 0.8);
    }
}
