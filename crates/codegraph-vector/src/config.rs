//! Configuration for Qdrant vector storage
//!
//! Defines vector dimensions, distance metrics, and connection settings.

use serde::{Deserialize, Serialize};

/// Default vector size for OpenAI embeddings
pub const DEFAULT_VECTOR_SIZE: u64 = 1536;

/// Default distance metric
pub const DEFAULT_DISTANCE: Distance = Distance::Cosine;

/// Distance metrics for vector similarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Distance {
    /// Cosine similarity (default, best for text embeddings)
    Cosine,
    /// Euclidean distance
    Euclid,
    /// Dot product
    Dot,
    /// Manhattan distance
    Manhattan,
}

impl Default for Distance {
    fn default() -> Self {
        Self::Cosine
    }
}

impl Distance {
    /// Convert to Qdrant distance type
    pub fn to_qdrant(&self) -> qdrant_client::qdrant::Distance {
        match self {
            Distance::Cosine => qdrant_client::qdrant::Distance::Cosine,
            Distance::Euclid => qdrant_client::qdrant::Distance::Euclid,
            Distance::Dot => qdrant_client::qdrant::Distance::Dot,
            Distance::Manhattan => qdrant_client::qdrant::Distance::Manhattan,
        }
    }
}

/// Vector configuration for collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    /// Vector dimension size (default: 1536 for OpenAI)
    pub size: u64,
    /// Distance metric (default: Cosine)
    pub distance: Distance,
    /// Enable on-disk storage for large collections
    pub on_disk: bool,
    /// HNSW index parameters
    pub hnsw_config: Option<HnswConfig>,
}

impl Default for VectorConfig {
    fn default() -> Self {
        Self {
            size: DEFAULT_VECTOR_SIZE,
            distance: DEFAULT_DISTANCE,
            on_disk: false,
            hnsw_config: Some(HnswConfig::default()),
        }
    }
}

impl VectorConfig {
    /// Create config for OpenAI text-embedding-3-small (1536 dimensions)
    pub fn openai_small() -> Self {
        Self::default()
    }

    /// Create config for OpenAI text-embedding-3-large (3072 dimensions)
    pub fn openai_large() -> Self {
        Self {
            size: 3072,
            ..Default::default()
        }
    }

    /// Create config optimized for large collections (100k+ vectors)
    pub fn large_scale() -> Self {
        Self {
            size: DEFAULT_VECTOR_SIZE,
            distance: Distance::Cosine,
            on_disk: true,
            hnsw_config: Some(HnswConfig::large_scale()),
        }
    }
}

/// HNSW index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfig {
    /// Number of edges per node (default: 16)
    pub m: u64,
    /// Size of the dynamic candidate list (default: 100)
    pub ef_construct: u64,
    /// Store vectors on disk
    pub on_disk: bool,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construct: 100,
            on_disk: false,
        }
    }
}

impl HnswConfig {
    /// Configuration optimized for large collections
    pub fn large_scale() -> Self {
        Self {
            m: 32,
            ef_construct: 200,
            on_disk: true,
        }
    }
}

/// Qdrant connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    /// Qdrant server URL
    pub url: String,
    /// API key (optional)
    pub api_key: Option<String>,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Vector configuration
    pub vector_config: VectorConfig,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6334".to_string(),
            api_key: None,
            timeout_secs: 30,
            vector_config: VectorConfig::default(),
        }
    }
}

impl QdrantConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string()),
            api_key: std::env::var("QDRANT_API_KEY").ok(),
            timeout_secs: std::env::var("QDRANT_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            vector_config: VectorConfig::default(),
        }
    }

    /// Set URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set vector config
    pub fn with_vector_config(mut self, config: VectorConfig) -> Self {
        self.vector_config = config;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_vector_config() {
        let config = VectorConfig::default();
        assert_eq!(config.size, 1536);
        assert_eq!(config.distance, Distance::Cosine);
        assert!(!config.on_disk);
    }

    #[test]
    fn test_openai_configs() {
        let small = VectorConfig::openai_small();
        assert_eq!(small.size, 1536);

        let large = VectorConfig::openai_large();
        assert_eq!(large.size, 3072);
    }

    #[test]
    fn test_large_scale_config() {
        let config = VectorConfig::large_scale();
        assert!(config.on_disk);
        assert!(config.hnsw_config.as_ref().unwrap().on_disk);
    }

    #[test]
    fn test_qdrant_config_builder() {
        let config = QdrantConfig::default()
            .with_url("http://qdrant:6334")
            .with_api_key("secret");

        assert_eq!(config.url, "http://qdrant:6334");
        assert_eq!(config.api_key, Some("secret".to_string()));
    }

    #[test]
    fn test_distance_to_qdrant() {
        assert_eq!(
            Distance::Cosine.to_qdrant() as i32,
            qdrant_client::qdrant::Distance::Cosine as i32
        );
    }
}
