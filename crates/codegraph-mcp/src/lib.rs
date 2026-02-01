//! CodeGraph MCP - Model Context Protocol server
//!
//! Implements the MCP protocol for AI assistants to interact with CodeGraph.
//!
//! ## Features
//!
//! - JSON-RPC 2.0 over stdio transport
//! - Tools for UI component extraction, querying, and generation
//! - RLKGF feedback integration
//! - Resources for metrics and recent generations
//!
//! ## Tools
//!
//! - `extract_snippet`: Extract UI elements from HTML/CSS/JS
//! - `query_ui`: Search for UI components with NARS reasoning
//! - `generate_code`: Generate UI code from natural language
//! - `give_feedback`: RLKGF feedback for learning
//! - `get_graph_stats`: Knowledge graph statistics
//!
//! ## Resources
//!
//! - `codegraph://metrics`: RLKGF metrics
//! - `codegraph://recent`: Recent generations
//!
//! ## Usage
//!
//! ```ignore
//! use codegraph_mcp::run_stdio;
//!
//! #[tokio::main]
//! async fn main() {
//!     run_stdio().await.unwrap();
//! }
//! ```

pub mod protocol;
pub mod resources;
pub mod server;
pub mod tools;

pub use protocol::{error_codes, Request, Response};
pub use server::run_stdio;
