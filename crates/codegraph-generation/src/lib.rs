//! CodeGraph Generation - LLM-powered vanilla code generation
//!
//! Generates HTML5/CSS3/ES6+ code from graph retrieval results using GPT-4o.

pub mod generator;
pub mod parser;
pub mod prompt;
pub mod templates;

pub use generator::{GenerationRequest, GenerationResult, VanillaCodeGenerator};
pub use parser::{CodeParser, ParsedCode};
pub use prompt::SimilarElement;
pub use templates::TemplateEngine;
