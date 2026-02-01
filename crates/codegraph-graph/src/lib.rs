//! CodeGraph Graph - Neo4j graph storage and queries
//!
//! Provides persistence and semantic queries for UI component graphs.

pub mod entities;
pub mod relations;
pub mod repository;
pub mod schema;

pub use entities::{DesignSystem, Snippet, SnippetSummary, UIElement};
pub use relations::RelationManager;
pub use repository::Neo4jRepository;
pub use schema::SchemaManager;
