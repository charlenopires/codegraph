//! CodeGraph Vector - Qdrant integration for UI component embeddings
//!
//! This crate provides vector storage and similarity search capabilities
//! for UI component embeddings using Qdrant vector database.
//!
//! ## Features
//!
//! - Separate collections per design system (Material, Tailwind, Chakra, Bootstrap, Custom)
//! - OpenAI-compatible embeddings (1536 dimensions, Cosine distance)
//! - Payload indexing for fast filtered searches
//! - Redis caching with 1-hour TTL
//! - Supports 100k+ embeddings

pub mod collections;
pub mod config;
pub mod error;
pub mod models;
pub mod cache;
pub mod repository;

pub use collections::{Collection, COLLECTIONS};
pub use config::{QdrantConfig, VectorConfig};
pub use error::VectorError;
pub use models::{EmbeddingPoint, SearchFilter, SearchResult};
pub use repository::QdrantRepository;
pub use cache::EmbeddingCache;
