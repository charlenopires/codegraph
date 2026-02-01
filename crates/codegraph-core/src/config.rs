//! Centralized configuration from environment variables
//!
//! All configuration is loaded from environment variables with sensible defaults.
//! See the `Config` struct for all available options.

use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    // Server
    pub server: ServerConfig,
    // Database
    pub neo4j: Neo4jConfig,
    pub qdrant: QdrantConfig,
    pub redis: RedisConfig,
    // External APIs
    pub openai: OpenAIConfig,
    // ONA/NARS
    pub ona: OnaConfig,
    // Rate limiting
    pub rate_limit: RateLimitConfig,
    // Logging
    pub log_level: String,
    pub log_format: String,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
}

/// Neo4j database configuration
#[derive(Debug, Clone)]
pub struct Neo4jConfig {
    pub uri: String,
    pub user: String,
    pub password: String,
    pub max_connections: u32,
}

/// Qdrant vector database configuration
#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub collection: String,
    pub vector_size: u64,
}

/// Redis configuration
#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: Option<String>,
}

/// OpenAI API configuration
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub embedding_model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

/// ONA/NARS reasoning configuration
#[derive(Debug, Clone)]
pub struct OnaConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub inference_cycles: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u64,
    pub window_seconds: u64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            server: ServerConfig::from_env(),
            neo4j: Neo4jConfig::from_env(),
            qdrant: QdrantConfig::from_env(),
            redis: RedisConfig::from_env(),
            openai: OpenAIConfig::from_env(),
            ona: OnaConfig::from_env(),
            rate_limit: RateLimitConfig::from_env(),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            log_format: env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string()),
        }
    }

    /// Validate configuration and return errors if invalid
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push("SERVER_PORT must be > 0".to_string());
        }

        if self.neo4j.uri.is_empty() {
            errors.push("NEO4J_URI is required".to_string());
        }

        if self.qdrant.url.is_empty() {
            errors.push("QDRANT_URL is required".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            request_timeout_secs: env::var("REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(30),
        }
    }
}

impl Neo4jConfig {
    pub fn from_env() -> Self {
        Self {
            uri: env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string()),
            user: env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string()),
            password: env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "codegraph123".to_string()),
            max_connections: env::var("NEO4J_MAX_CONNECTIONS")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(50),
        }
    }
}

impl QdrantConfig {
    pub fn from_env() -> Self {
        Self {
            url: env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string()),
            collection: env::var("QDRANT_COLLECTION")
                .unwrap_or_else(|_| "ui_elements".to_string()),
            vector_size: env::var("QDRANT_VECTOR_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1536),
        }
    }
}

impl RedisConfig {
    pub fn from_env() -> Self {
        Self {
            url: env::var("REDIS_URL").ok(),
        }
    }
}

impl OpenAIConfig {
    pub fn from_env() -> Self {
        Self {
            api_key: env::var("OPENAI_API_KEY").ok(),
            model: env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
            embedding_model: env::var("OPENAI_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string()),
            max_tokens: env::var("OPENAI_MAX_TOKENS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(4096),
            temperature: env::var("OPENAI_TEMPERATURE")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(0.7),
        }
    }
}

impl OnaConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env::var("CODEGRAPH_ONA_ENABLED")
                .map(|v| v.to_lowercase() != "false" && v != "0")
                .unwrap_or(true),
            host: env::var("ONA_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("ONA_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(50000),
            inference_cycles: env::var("ONA_INFERENCE_CYCLES")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(100),
        }
    }
}

impl RateLimitConfig {
    pub fn from_env() -> Self {
        Self {
            requests_per_minute: env::var("RATE_LIMIT_RPM")
                .ok()
                .and_then(|r| r.parse().ok())
                .unwrap_or(100),
            window_seconds: env::var("RATE_LIMIT_WINDOW_SECS")
                .ok()
                .and_then(|w| w.parse().ok())
                .unwrap_or(60),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::from_env();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.neo4j.max_connections, 50);
        assert_eq!(config.rate_limit.requests_per_minute, 100);
    }
}
