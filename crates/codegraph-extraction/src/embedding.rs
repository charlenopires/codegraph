//! Embedding Generator - generates vector embeddings for UI elements

use async_openai::{
    config::OpenAIConfig,
    types::{CreateEmbeddingRequestArgs, EmbeddingInput},
    Client,
};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, warn};

use crate::ontology::OntologyMapping;

/// Embedding model configuration
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub model: String,
    pub dimensions: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model: "text-embedding-3-large".to_string(),
            dimensions: 1536,
        }
    }
}

/// Generated embedding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    pub embedding: Vec<f32>,
    pub text_used: String,
    pub model: String,
    pub dimensions: usize,
}

/// Embedding generator using OpenAI
pub struct EmbeddingGenerator {
    client: Option<Client<OpenAIConfig>>,
    config: EmbeddingConfig,
}

impl Default for EmbeddingGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingGenerator {
    pub fn new() -> Self {
        let client = if env::var("OPENAI_API_KEY").is_ok() {
            Some(Client::new())
        } else {
            warn!("OPENAI_API_KEY not set, embedding generation will use fallback");
            None
        };

        Self {
            client,
            config: EmbeddingConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: EmbeddingConfig) -> Self {
        let client = if env::var("OPENAI_API_KEY").is_ok() {
            Some(Client::new())
        } else {
            warn!("OPENAI_API_KEY not set, embedding generation will use fallback");
            None
        };

        Self { client, config }
    }

    /// Generate embedding for text
    pub async fn generate_text_embedding(&self, text: &str) -> anyhow::Result<EmbeddingResult> {
        if let Some(client) = &self.client {
            let request = CreateEmbeddingRequestArgs::default()
                .model(&self.config.model)
                .input(EmbeddingInput::String(text.to_string()))
                .dimensions(self.config.dimensions as u32)
                .build()?;

            let response = client.embeddings().create(request).await?;

            if let Some(data) = response.data.first() {
                debug!("Generated embedding with {} dimensions", data.embedding.len());
                return Ok(EmbeddingResult {
                    embedding: data.embedding.clone(),
                    text_used: text.to_string(),
                    model: self.config.model.clone(),
                    dimensions: data.embedding.len(),
                });
            }
        }

        // Fallback: generate deterministic pseudo-embedding
        Ok(self.generate_fallback_embedding(text))
    }

    /// Generate embedding for UI element from ontology mapping
    pub async fn generate_element_embedding(
        &self,
        mapping: &OntologyMapping,
        html_template: Option<&str>,
    ) -> anyhow::Result<EmbeddingResult> {
        let text = self.build_embedding_text(mapping, html_template);
        self.generate_text_embedding(&text).await
    }

    /// Build text representation for embedding
    fn build_embedding_text(&self, mapping: &OntologyMapping, html_template: Option<&str>) -> String {
        let mut parts = Vec::new();

        // Design system
        if let Some(ds) = mapping.design_system {
            parts.push(format!("design system: {}", crate::design_system::DesignSystemType::as_str(&ds)));
        }

        // Categories
        let categories: Vec<&str> = mapping.categories_used.iter().map(|c| crate::ontology::UICategory::as_str(c)).collect();
        if !categories.is_empty() {
            parts.push(format!("categories: {}", categories.join(", ")));
        }

        // Element details
        for element in &mapping.elements {
            parts.push(format!("element type: {}", element.element_type));
            if !element.classes.is_empty() {
                parts.push(format!("classes: {}", element.classes.join(" ")));
            }
            if element.has_interactivity {
                parts.push("interactive: yes".to_string());
            }
        }

        // Design tokens
        for (name, value, category) in &mapping.design_tokens {
            parts.push(format!("token {} ({:?}): {}", name, category, value));
        }

        // HTML template
        if let Some(html) = html_template {
            parts.push(format!("html: {}", html));
        }

        parts.join(" | ")
    }

    /// Generate fallback embedding when API is unavailable
    fn generate_fallback_embedding(&self, text: &str) -> EmbeddingResult {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Generate deterministic embedding based on text hash
        let mut embedding = vec![0.0f32; self.config.dimensions];

        // Use multiple hash seeds for variety
        for (i, chunk) in embedding.chunks_mut(64).enumerate() {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            i.hash(&mut hasher);
            let hash = hasher.finish();

            for (j, val) in chunk.iter_mut().enumerate() {
                let seed = hash.wrapping_add(j as u64);
                // Generate pseudo-random value between -1 and 1
                *val = ((seed % 10000) as f32 / 5000.0) - 1.0;
            }
        }

        // Normalize the vector
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }

        EmbeddingResult {
            embedding,
            text_used: text.to_string(),
            model: "fallback-hash".to_string(),
            dimensions: self.config.dimensions,
        }
    }

    /// Batch generate embeddings for multiple texts
    pub async fn generate_batch(&self, texts: &[String]) -> anyhow::Result<Vec<EmbeddingResult>> {
        if let Some(client) = &self.client {
            let request = CreateEmbeddingRequestArgs::default()
                .model(&self.config.model)
                .input(EmbeddingInput::StringArray(texts.to_vec()))
                .dimensions(self.config.dimensions as u32)
                .build()?;

            let response = client.embeddings().create(request).await?;

            let results: Vec<_> = response
                .data
                .into_iter()
                .zip(texts.iter())
                .map(|(data, text)| EmbeddingResult {
                    embedding: data.embedding,
                    text_used: text.clone(),
                    model: self.config.model.clone(),
                    dimensions: self.config.dimensions,
                })
                .collect();

            return Ok(results);
        }

        // Fallback: generate for each text individually
        Ok(texts
            .iter()
            .map(|text| self.generate_fallback_embedding(text))
            .collect())
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }

        dot / (mag_a * mag_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_embedding() {
        let generator = EmbeddingGenerator::new();
        let result = generator.generate_fallback_embedding("test button component");

        assert_eq!(result.dimensions, 1536);
        assert_eq!(result.embedding.len(), 1536);
        assert_eq!(result.model, "fallback-hash");

        // Check normalization
        let magnitude: f32 = result.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_deterministic_fallback() {
        let generator = EmbeddingGenerator::new();
        let result1 = generator.generate_fallback_embedding("same text");
        let result2 = generator.generate_fallback_embedding("same text");

        assert_eq!(result1.embedding, result2.embedding);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        assert!((EmbeddingGenerator::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!(EmbeddingGenerator::cosine_similarity(&a, &c).abs() < 0.001);
    }
}
