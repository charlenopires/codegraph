//! CodeGraph Reasoning - OpenNARS for Applications (ONA) integration
//!
//! Implements NARS reasoning pipeline for semantic query understanding.

pub mod inference;
pub mod llm_translator;
pub mod narsese;
pub mod ona;
pub mod pipeline;

pub use inference::InferenceEngine;
pub use llm_translator::LlmNarseseTranslator;
pub use narsese::{NarseseStatement, NarseseTranslator};
pub use ona::OnaClient;
pub use pipeline::{ReasoningPipeline, ReasoningResult};
