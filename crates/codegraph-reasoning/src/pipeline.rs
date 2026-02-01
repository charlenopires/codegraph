//! Reasoning pipeline - orchestrates NARS-based query understanding

use std::env;

use tracing::{debug, info, warn};

use crate::narsese::{extract_search_terms, parse_ona_response, NarseseStatement, NarseseTranslator};
use crate::ona::OnaClient;

/// Result of the reasoning pipeline
#[derive(Debug, Clone)]
pub struct ReasoningResult {
    /// Original query
    pub query: String,
    /// Detected intent (create, find, modify)
    pub intent: String,
    /// Translated Narsese statements
    pub input_statements: Vec<NarseseStatement>,
    /// Derived statements from NARS inference
    pub derived_statements: Vec<NarseseStatement>,
    /// Extracted search terms for retrieval
    pub search_terms: Vec<String>,
    /// Raw ONA output for debugging
    pub raw_output: String,
}

/// NARS reasoning pipeline for semantic query understanding
///
/// Pipeline: Query NL → translate_to_narsese → load_ontology → input_statements → step(100) → parse_responses
///
/// # Fallback Mode
///
/// When ONA is unavailable or disabled via `CODEGRAPH_ONA_ENABLED=false`,
/// the pipeline operates in offline mode using only rule-based translation.
/// This provides basic functionality without NARS inference.
pub struct ReasoningPipeline {
    translator: NarseseTranslator,
    ona: OnaClient,
    inference_cycles: u32,
    ontology_loaded: bool,
    ona_enabled: bool,
}

impl ReasoningPipeline {
    /// Create a new reasoning pipeline
    ///
    /// ONA integration can be disabled by setting `CODEGRAPH_ONA_ENABLED=false`
    pub fn new() -> Self {
        let ona_enabled = env::var("CODEGRAPH_ONA_ENABLED")
            .map(|v| v.to_lowercase() != "false" && v != "0")
            .unwrap_or(true);

        if !ona_enabled {
            warn!("ONA integration disabled via CODEGRAPH_ONA_ENABLED=false, using offline mode");
        }

        Self {
            translator: NarseseTranslator::new(),
            ona: OnaClient::new(),
            inference_cycles: 100,
            ontology_loaded: false,
            ona_enabled,
        }
    }

    /// Check if ONA is enabled
    pub fn is_ona_enabled(&self) -> bool {
        self.ona_enabled
    }

    /// Set number of inference cycles
    pub fn with_inference_cycles(mut self, cycles: u32) -> Self {
        self.inference_cycles = cycles;
        self
    }

    /// Initialize the pipeline (load ontology)
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        if !self.ontology_loaded {
            info!("Loading UI ontology into ONA");
            self.ona.load_ontology()?;
            self.ontology_loaded = true;
        }
        Ok(())
    }

    /// Process a natural language query through the full pipeline
    ///
    /// If ONA is disabled or unavailable, automatically falls back to offline mode.
    pub fn process(&mut self, query: &str) -> anyhow::Result<ReasoningResult> {
        info!("Processing query: {}", query);

        // Check if ONA is disabled
        if !self.ona_enabled {
            debug!("ONA disabled, using offline processing");
            return Ok(self.process_offline(query));
        }

        // Step 1: Translate to Narsese
        let input_statements = self.translator.translate(query);
        debug!(
            "Translated to {} Narsese statements",
            input_statements.len()
        );

        // Detect intent
        let intent = self.translator.detect_intent(query).to_string();
        debug!("Detected intent: {}", intent);

        // Step 2: Ensure ontology is loaded
        if !self.ontology_loaded {
            if let Err(e) = self.initialize() {
                warn!("Failed to load ONA ontology: {}, falling back to offline mode", e);
                return Ok(self.process_offline(query));
            }
        }

        // Step 3: Input statements to ONA
        if let Err(e) = self.ona.input_statements(&input_statements) {
            warn!("Failed to input statements to ONA: {}, falling back to offline mode", e);
            return Ok(self.process_offline(query));
        }

        // Step 4: Run inference cycles
        let raw_output = match self.ona.step(self.inference_cycles) {
            Ok(output) => output,
            Err(e) => {
                warn!("ONA inference failed: {}, falling back to offline mode", e);
                return Ok(self.process_offline(query));
            }
        };
        debug!("ONA output: {} chars", raw_output.len());

        // Step 5: Parse responses
        let derived_statements = parse_ona_response(&raw_output);
        debug!("Derived {} statements", derived_statements.len());

        // Step 6: Extract search terms from all statements
        let all_statements: Vec<_> = input_statements
            .iter()
            .chain(derived_statements.iter())
            .cloned()
            .collect();
        let search_terms = extract_search_terms(&all_statements);
        debug!("Extracted {} search terms", search_terms.len());

        Ok(ReasoningResult {
            query: query.to_string(),
            intent,
            input_statements,
            derived_statements,
            search_terms,
            raw_output,
        })
    }

    /// Process without ONA (fallback mode using only translation)
    pub fn process_offline(&self, query: &str) -> ReasoningResult {
        let input_statements = self.translator.translate(query);
        let intent = self.translator.detect_intent(query).to_string();
        let search_terms = extract_search_terms(&input_statements);

        ReasoningResult {
            query: query.to_string(),
            intent,
            input_statements,
            derived_statements: vec![],
            search_terms,
            raw_output: String::new(),
        }
    }
}

impl Default for ReasoningPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_processing() {
        let pipeline = ReasoningPipeline::new();
        let result = pipeline.process_offline("create a responsive button");

        assert_eq!(result.intent, "create");
        assert!(result.search_terms.contains(&"button".to_string()));
        assert!(result.search_terms.contains(&"responsive".to_string()));
    }
}
