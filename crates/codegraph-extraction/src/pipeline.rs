//! Extraction Pipeline - orchestrates full extraction with <500ms target

use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::css::{CssParser, CssStructure};
use crate::design_system::{DesignSystemDetector, DetectionResult};
use crate::embedding::{EmbeddingGenerator, EmbeddingResult};
use crate::html::{HtmlParser, HtmlStructure};
use crate::javascript::{JsParser, JsStructure};
use crate::narsese_gen::{NarseseGenerator, NarseseKB};
use crate::ontology::{OntologyMapper, OntologyMapping};

/// Complete extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Parsed HTML structure
    pub html: HtmlStructure,
    /// Parsed CSS structure
    pub css: CssStructure,
    /// Parsed JavaScript structure
    pub js: JsStructure,
    /// Detected design system
    pub design_system: DetectionResult,
    /// Ontology mapping
    pub ontology: OntologyMapping,
    /// Generated Narsese statements
    pub narsese: NarseseKB,
    /// Generated embedding (if requested)
    pub embedding: Option<EmbeddingResult>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Input code for extraction
#[derive(Debug, Clone)]
pub struct ExtractionInput {
    pub html: String,
    pub css: Option<String>,
    pub js: Option<String>,
}

impl ExtractionInput {
    pub fn new(html: impl Into<String>) -> Self {
        Self {
            html: html.into(),
            css: None,
            js: None,
        }
    }

    pub fn with_css(mut self, css: impl Into<String>) -> Self {
        self.css = Some(css.into());
        self
    }

    pub fn with_js(mut self, js: impl Into<String>) -> Self {
        self.js = Some(js.into());
        self
    }
}

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Generate embeddings (requires API key)
    pub generate_embeddings: bool,
    /// Target processing time in ms
    pub target_time_ms: u64,
    /// Warn if exceeded
    pub warn_on_slow: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            generate_embeddings: true,
            target_time_ms: 500,
            warn_on_slow: true,
        }
    }
}

/// Extraction pipeline - orchestrates all extraction components
pub struct ExtractionPipeline {
    html_parser: HtmlParser,
    css_parser: CssParser,
    js_parser: JsParser,
    design_detector: DesignSystemDetector,
    ontology_mapper: OntologyMapper,
    narsese_generator: NarseseGenerator,
    embedding_generator: EmbeddingGenerator,
    config: PipelineConfig,
}

impl Default for ExtractionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtractionPipeline {
    pub fn new() -> Self {
        Self {
            html_parser: HtmlParser::new(),
            css_parser: CssParser::new(),
            js_parser: JsParser::new(),
            design_detector: DesignSystemDetector::new(),
            ontology_mapper: OntologyMapper::new(),
            narsese_generator: NarseseGenerator::new(),
            embedding_generator: EmbeddingGenerator::new(),
            config: PipelineConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: PipelineConfig) -> Self {
        Self {
            html_parser: HtmlParser::new(),
            css_parser: CssParser::new(),
            js_parser: JsParser::new(),
            design_detector: DesignSystemDetector::new(),
            ontology_mapper: OntologyMapper::new(),
            narsese_generator: NarseseGenerator::new(),
            embedding_generator: EmbeddingGenerator::new(),
            config,
        }
    }

    /// Run full extraction pipeline
    pub async fn extract(&mut self, input: ExtractionInput) -> anyhow::Result<ExtractionResult> {
        let start = Instant::now();

        // Phase 1: Parse all input (parallel would be ideal, but parsers are mutable)
        debug!("Phase 1: Parsing input");
        let html = self.html_parser.parse(&input.html)?;
        let css = input
            .css
            .as_ref()
            .map(|c| self.css_parser.parse(c))
            .transpose()?
            .unwrap_or_else(|| CssStructure {
                rules: vec![],
                design_tokens: vec![],
                properties: vec![],
                selectors: vec![],
            });
        let js = input
            .js
            .as_ref()
            .map(|j| self.js_parser.parse(j))
            .transpose()?
            .unwrap_or_else(|| JsStructure {
                functions: vec![],
                event_handlers: vec![],
                imports: vec![],
                variables: vec![],
                dom_calls: vec![],
            });

        let parse_time = start.elapsed().as_millis();
        debug!("Parsing completed in {}ms", parse_time);

        // Phase 2: Design system detection
        debug!("Phase 2: Design system detection");
        let design_system = self.design_detector.detect_from_content(
            &input.html,
            input.css.as_deref().unwrap_or(""),
            input.js.as_deref().unwrap_or(""),
        );
        debug!(
            "Detected design system: {:?} (confidence: {:.2})",
            design_system.design_system, design_system.confidence
        );

        // Phase 3: Ontology mapping
        debug!("Phase 3: Ontology mapping");
        let ontology = self.ontology_mapper.map_full(&html, &css, &js, &design_system);
        debug!("Mapped {} elements to ontology", ontology.elements.len());

        // Phase 4: Narsese generation
        debug!("Phase 4: Narsese generation");
        let narsese = self.narsese_generator.generate(&ontology);
        debug!("Generated {} Narsese statements", narsese.statements.len());

        // Phase 5: Embedding generation (optional, async)
        debug!("Phase 5: Embedding generation");
        let embedding = if self.config.generate_embeddings {
            match self
                .embedding_generator
                .generate_element_embedding(&ontology, Some(&input.html))
                .await
            {
                Ok(emb) => {
                    debug!("Generated embedding with {} dimensions", emb.dimensions);
                    Some(emb)
                }
                Err(e) => {
                    warn!("Embedding generation failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let processing_time_ms = start.elapsed().as_millis() as u64;

        // Check performance target
        if self.config.warn_on_slow && processing_time_ms > self.config.target_time_ms {
            warn!(
                "Extraction took {}ms, exceeding target of {}ms",
                processing_time_ms, self.config.target_time_ms
            );
        } else {
            info!("Extraction completed in {}ms", processing_time_ms);
        }

        Ok(ExtractionResult {
            html,
            css,
            js,
            design_system,
            ontology,
            narsese,
            embedding,
            processing_time_ms,
        })
    }

    /// Quick extraction without embeddings
    pub fn extract_sync(&mut self, input: ExtractionInput) -> anyhow::Result<ExtractionResult> {
        let start = Instant::now();

        let html = self.html_parser.parse(&input.html)?;
        let css = input
            .css
            .as_ref()
            .map(|c| self.css_parser.parse(c))
            .transpose()?
            .unwrap_or_else(|| CssStructure {
                rules: vec![],
                design_tokens: vec![],
                properties: vec![],
                selectors: vec![],
            });
        let js = input
            .js
            .as_ref()
            .map(|j| self.js_parser.parse(j))
            .transpose()?
            .unwrap_or_else(|| JsStructure {
                functions: vec![],
                event_handlers: vec![],
                imports: vec![],
                variables: vec![],
                dom_calls: vec![],
            });

        let design_system = self.design_detector.detect_from_content(
            &input.html,
            input.css.as_deref().unwrap_or(""),
            input.js.as_deref().unwrap_or(""),
        );

        let ontology = self.ontology_mapper.map_full(&html, &css, &js, &design_system);
        let narsese = self.narsese_generator.generate(&ontology);

        let processing_time_ms = start.elapsed().as_millis() as u64;

        if self.config.warn_on_slow && processing_time_ms > self.config.target_time_ms {
            warn!(
                "Sync extraction took {}ms, exceeding target of {}ms",
                processing_time_ms, self.config.target_time_ms
            );
        }

        Ok(ExtractionResult {
            html,
            css,
            js,
            design_system,
            ontology,
            narsese,
            embedding: None,
            processing_time_ms,
        })
    }

    /// Get design system detector reference
    pub fn design_detector(&self) -> &DesignSystemDetector {
        &self.design_detector
    }

    /// Get ontology mapper reference
    pub fn ontology_mapper(&self) -> &OntologyMapper {
        &self.ontology_mapper
    }

    /// Get narsese generator reference
    pub fn narsese_generator(&self) -> &NarseseGenerator {
        &self.narsese_generator
    }

    /// Get embedding generator reference
    pub fn embedding_generator(&self) -> &EmbeddingGenerator {
        &self.embedding_generator
    }
}

// Re-export main types for convenience
pub use crate::css::TokenCategory;
pub use crate::design_system::DesignSystemType;
pub use crate::ontology::UICategory;
pub use crate::narsese_gen::TruthValue;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_extraction() {
        let mut pipeline = ExtractionPipeline::new();

        let input = ExtractionInput::new(
            r#"<button class="btn btn-primary p-4 rounded-lg">Click me</button>"#,
        )
        .with_css(
            r#"
            :root {
                --color-primary: #3b82f6;
            }
            .btn {
                background: var(--color-primary);
            }
        "#,
        );

        let result = pipeline.extract_sync(input).unwrap();

        assert!(!result.html.elements.is_empty());
        assert!(!result.ontology.elements.is_empty());
        assert!(!result.narsese.statements.is_empty());
        assert!(result.processing_time_ms < 1000); // Should be well under 1 second
    }

    #[test]
    fn test_extraction_performance() {
        let mut pipeline = ExtractionPipeline::with_config(PipelineConfig {
            generate_embeddings: false,
            target_time_ms: 500,
            warn_on_slow: false,
        });

        let input = ExtractionInput::new(r##"
            <div class="container mx-auto p-4">
                <nav class="navbar bg-white shadow-md">
                    <a href="#" class="brand">Logo</a>
                    <ul class="nav-menu">
                        <li><a href="#">Home</a></li>
                        <li><a href="#">About</a></li>
                        <li><a href="#">Contact</a></li>
                    </ul>
                </nav>
                <main class="grid grid-cols-3 gap-4 mt-8">
                    <div class="card p-4 rounded-lg shadow">
                        <h2 class="text-xl font-bold">Card 1</h2>
                        <p class="text-gray-600">Description</p>
                        <button class="btn btn-primary mt-4">Action</button>
                    </div>
                </main>
            </div>
        "##);

        let result = pipeline.extract_sync(input).unwrap();

        // Should complete in under 500ms for typical UI snippets
        assert!(
            result.processing_time_ms < 500,
            "Processing took {}ms, expected < 500ms",
            result.processing_time_ms
        );
    }

    #[tokio::test]
    async fn test_async_extraction() {
        let mut pipeline = ExtractionPipeline::with_config(PipelineConfig {
            generate_embeddings: true, // Will use fallback if no API key
            target_time_ms: 500,
            warn_on_slow: false,
        });

        let input = ExtractionInput::new(
            r#"<button class="btn btn-primary">Click me</button>"#,
        );

        let result = pipeline.extract(input).await.unwrap();

        // Embedding should be generated (fallback if no API key)
        assert!(result.embedding.is_some());
    }
}
