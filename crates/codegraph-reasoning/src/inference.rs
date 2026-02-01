//! Inference engine - executes NARS inference cycles and collects results

use tracing::{debug, info};

use crate::narsese::{parse_ona_response, NarseseStatement};
use crate::ona::OnaClient;

/// Result of inference execution
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Answer statements (direct query responses)
    pub answers: Vec<NarseseStatement>,
    /// Derived statements (inferred knowledge)
    pub derived: Vec<NarseseStatement>,
    /// Number of cycles executed
    pub cycles: u32,
    /// Raw ONA output
    pub raw_output: String,
}

/// NARS inference engine with configurable cycles
pub struct InferenceEngine {
    ona: OnaClient,
    default_cycles: u32,
    ontology_loaded: bool,
}

impl InferenceEngine {
    /// Create a new inference engine with 100 cycle default
    pub fn new() -> Self {
        Self {
            ona: OnaClient::new(),
            default_cycles: 100,
            ontology_loaded: false,
        }
    }

    /// Create with custom ONA client
    pub fn with_client(ona: OnaClient) -> Self {
        Self {
            ona,
            default_cycles: 100,
            ontology_loaded: false,
        }
    }

    /// Set default inference cycles
    pub fn with_cycles(mut self, cycles: u32) -> Self {
        self.default_cycles = cycles;
        self
    }

    /// Get the underlying ONA client
    pub fn client(&self) -> &OnaClient {
        &self.ona
    }

    /// Load the UI ontology
    pub fn load_ontology(&mut self) -> anyhow::Result<()> {
        if !self.ontology_loaded {
            info!("Loading UI ontology");
            self.ona.load_ontology()?;
            self.ontology_loaded = true;
        }
        Ok(())
    }

    /// Input statements and run inference
    pub fn infer(&self, statements: &[NarseseStatement]) -> anyhow::Result<InferenceResult> {
        self.infer_with_cycles(statements, self.default_cycles)
    }

    /// Input statements and run inference with specific cycle count
    pub fn infer_with_cycles(
        &self,
        statements: &[NarseseStatement],
        cycles: u32,
    ) -> anyhow::Result<InferenceResult> {
        info!(
            "Running inference with {} statements, {} cycles",
            statements.len(),
            cycles
        );

        // Input all statements
        self.ona.input_statements(statements)?;

        // Run inference cycles
        let raw_output = self.ona.step(cycles)?;
        debug!("ONA output: {} bytes", raw_output.len());

        // Parse responses
        let all_statements = parse_ona_response(&raw_output);

        // Separate answers from derived
        let (answers, derived): (Vec<_>, Vec<_>) = all_statements
            .into_iter()
            .partition(|s| raw_output.contains(&format!("Answer: {}", s.statement)));

        info!(
            "Inference complete: {} answers, {} derived",
            answers.len(),
            derived.len()
        );

        Ok(InferenceResult {
            answers,
            derived,
            cycles,
            raw_output,
        })
    }

    /// Query for specific knowledge
    pub fn query(&self, question: &str) -> anyhow::Result<InferenceResult> {
        info!("Querying: {}", question);

        let raw_output = self.ona.query(question)?;
        let all_statements = parse_ona_response(&raw_output);

        let (answers, derived): (Vec<_>, Vec<_>) = all_statements
            .into_iter()
            .partition(|s| raw_output.contains(&format!("Answer: {}", s.statement)));

        Ok(InferenceResult {
            answers,
            derived,
            cycles: 0,
            raw_output,
        })
    }

    /// Reset the inference engine state
    pub fn reset(&self) -> anyhow::Result<()> {
        info!("Resetting inference engine");
        self.ona.reset()?;
        Ok(())
    }

    /// Flush buffers
    pub fn flush(&self) -> anyhow::Result<()> {
        self.ona.flush()?;
        Ok(())
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}
