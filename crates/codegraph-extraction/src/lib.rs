//! CodeGraph Extraction - AST parsing and entity extraction
//!
//! Extracts ontological entities from UI code snippets using tree-sitter.

pub mod css;
pub mod design_system;
pub mod embedding;
pub mod html;
pub mod javascript;
pub mod narsese_gen;
pub mod ontology;
pub mod pipeline;

pub use design_system::DesignSystemDetector;
pub use embedding::EmbeddingGenerator;
pub use narsese_gen::NarseseGenerator;
pub use ontology::OntologyMapper;
pub use pipeline::{ExtractionInput, ExtractionPipeline, ExtractionResult};
