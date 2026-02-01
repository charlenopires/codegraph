//! CodeGraph CLI - Main entry point
//!
//! This is the main entry point for the CodeGraph system.
//! It initializes all components and starts the requested service.

use std::sync::Arc;

use clap::Parser;
use tracing::{error, info, warn};

use codegraph_extraction::embedding::EmbeddingGenerator;
use codegraph_generation::VanillaCodeGenerator;
use codegraph_graph::Neo4jRepository;
use codegraph_retrieval::HybridRetriever;
use codegraph_vector::{QdrantConfig, QdrantRepository};
use codegraph_web::AppState;

#[derive(Parser)]
#[command(name = "codegraph")]
#[command(about = "GraphRAG + NARS system for UI code generation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Start the web server
    Serve {
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Run MCP server over stdio
    Mcp,
    /// Run benchmark suite
    Benchmark,
}

/// Validate that all services are reachable before serving requests
async fn validate_connections(
    state: &AppState,
    qdrant: Option<&Arc<QdrantRepository>>,
) -> anyhow::Result<()> {
    // Validate Neo4j by running a simple count query
    match state.repository.count().await {
        Ok(count) => info!("Neo4j validated: {} UI elements in database", count),
        Err(e) => {
            error!("Neo4j validation failed: {}", e);
            return Err(anyhow::anyhow!("Neo4j connection validation failed: {}", e));
        }
    }

    // Validate Qdrant if available
    if let Some(qdrant_repo) = qdrant {
        match qdrant_repo.collection_info("ui_elements").await {
            Ok(info) => info!(
                "Qdrant validated: {} points in ui_elements collection",
                info.points_count
            ),
            Err(e) => {
                warn!("Qdrant validation warning: {}. Vector search may not work.", e);
                // Don't fail - Qdrant is optional
            }
        }
    } else {
        info!("Qdrant not configured - skipping validation");
    }

    // Validate OpenAI API key if set
    if std::env::var("OPENAI_API_KEY").is_ok() {
        info!("OpenAI API key configured");
    } else {
        warn!("OpenAI API key not set - using fallback generators");
    }

    info!("All service connections validated");
    Ok(())
}

/// Initialize all application components
async fn init_app_state() -> anyhow::Result<AppState> {
    info!("Initializing CodeGraph application state...");

    // 1. Initialize Neo4j repository
    info!("Connecting to Neo4j...");
    let neo4j_repository = match Neo4jRepository::new().await {
        Ok(repo) => {
            info!("Neo4j connected successfully");

            // Initialize schema (constraints + indexes)
            info!("Initializing Neo4j schema...");
            if let Err(e) = repo.initialize_schema().await {
                warn!("Failed to initialize Neo4j schema: {}. Continuing anyway.", e);
            } else {
                info!("Neo4j schema initialized successfully");
            }

            repo
        }
        Err(e) => {
            error!("Failed to connect to Neo4j: {}. Check NEO4J_URI, NEO4J_USER, NEO4J_PASSWORD", e);
            return Err(e);
        }
    };

    // 2. Initialize Qdrant repository and collections
    info!("Connecting to Qdrant...");
    let qdrant_repository = match QdrantRepository::new(QdrantConfig::default()).await {
        Ok(repo) => {
            info!("Qdrant connected successfully");

            // Initialize collections on startup
            info!("Initializing Qdrant collections...");
            if let Err(e) = repo.init_collections().await {
                warn!("Failed to initialize some Qdrant collections: {}. Continuing anyway.", e);
            } else {
                info!("Qdrant collections initialized successfully");
            }

            Some(Arc::new(repo))
        }
        Err(e) => {
            warn!("Failed to connect to Qdrant: {}. Vector search will be disabled.", e);
            None
        }
    };

    // 3. Initialize embedding generator
    let embedding_generator = Arc::new(EmbeddingGenerator::new());
    if std::env::var("OPENAI_API_KEY").is_err() {
        warn!("OPENAI_API_KEY not set. Embeddings will use fallback hash-based generator.");
    }

    // 4. Initialize code generator
    let generator = VanillaCodeGenerator::new();
    if std::env::var("OPENAI_API_KEY").is_err() {
        warn!("OPENAI_API_KEY not set. Code generation will use fallback templates.");
    }

    // 5. Create AppState with all components
    // Note: Retriever is configured later in AppState::new since Neo4j can't be cloned
    info!("Creating application state...");
    let retriever = HybridRetriever::new().with_embedding_generator(embedding_generator);

    // If Qdrant is available, configure it in the retriever
    let retriever = if let Some(ref qdrant) = qdrant_repository {
        retriever.with_qdrant(qdrant.clone())
    } else {
        retriever
    };

    let state = AppState::new(neo4j_repository, retriever, generator).await?;

    // 6. Validate connections before serving requests
    info!("Validating service connections...");
    validate_connections(&state, qdrant_repository.as_ref()).await?;

    info!("Application state initialized successfully");
    Ok(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { port } => {
            // Initialize all components
            let state = init_app_state().await?;

            // Start web server
            codegraph_web::serve(state, port).await?;
        }
        Commands::Mcp => {
            codegraph_mcp::run_stdio().await?;
        }
        Commands::Benchmark => {
            info!("Running benchmark suite...");
            codegraph_benchmark::run().await?;
        }
    }

    Ok(())
}
