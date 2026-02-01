//! Schema management - Constraints, indexes, and vector indexes for Neo4j

use neo4rs::Graph;
use tracing::{info, warn};

/// Manages Neo4j schema: constraints and indexes
pub struct SchemaManager {
    graph: Graph,
}

impl SchemaManager {
    pub fn new(graph: Graph) -> Self {
        Self { graph }
    }

    /// Initialize all schema elements (constraints + indexes)
    pub async fn initialize(&self) -> anyhow::Result<()> {
        info!("Initializing Neo4j schema...");

        self.create_constraints().await?;
        self.create_indexes().await?;
        self.create_vector_index().await?;
        self.create_fulltext_index().await?;

        info!("Neo4j schema initialized successfully");
        Ok(())
    }

    /// Create uniqueness constraints
    async fn create_constraints(&self) -> anyhow::Result<()> {
        // UIElement.id must be unique
        let ui_constraint = r#"
            CREATE CONSTRAINT ui_element_id_unique IF NOT EXISTS
            FOR (e:UIElement)
            REQUIRE e.id IS UNIQUE
        "#;

        // DesignSystem.name must be unique
        let ds_constraint = r#"
            CREATE CONSTRAINT design_system_name_unique IF NOT EXISTS
            FOR (d:DesignSystem)
            REQUIRE d.name IS UNIQUE
        "#;

        // Snippet.id must be unique
        let snippet_constraint = r#"
            CREATE CONSTRAINT snippet_id_unique IF NOT EXISTS
            FOR (s:Snippet)
            REQUIRE s.id IS UNIQUE
        "#;

        match self.graph.run(neo4rs::query(ui_constraint)).await {
            Ok(_) => info!("Created UIElement.id uniqueness constraint"),
            Err(e) => warn!("UIElement constraint may already exist: {}", e),
        }

        match self.graph.run(neo4rs::query(ds_constraint)).await {
            Ok(_) => info!("Created DesignSystem.name uniqueness constraint"),
            Err(e) => warn!("DesignSystem constraint may already exist: {}", e),
        }

        match self.graph.run(neo4rs::query(snippet_constraint)).await {
            Ok(_) => info!("Created Snippet.id uniqueness constraint"),
            Err(e) => warn!("Snippet constraint may already exist: {}", e),
        }

        Ok(())
    }

    /// Create property indexes for fast queries
    async fn create_indexes(&self) -> anyhow::Result<()> {
        let indexes = vec![
            // Index on category for filtering
            r#"
                CREATE INDEX ui_element_category IF NOT EXISTS
                FOR (e:UIElement)
                ON (e.category)
            "#,
            // Index on element_type
            r#"
                CREATE INDEX ui_element_type IF NOT EXISTS
                FOR (e:UIElement)
                ON (e.element_type)
            "#,
            // Index on design_system
            r#"
                CREATE INDEX ui_element_design_system IF NOT EXISTS
                FOR (e:UIElement)
                ON (e.design_system)
            "#,
            // Composite index for common query pattern
            r#"
                CREATE INDEX ui_element_category_type IF NOT EXISTS
                FOR (e:UIElement)
                ON (e.category, e.element_type)
            "#,
            // Snippet indexes
            r#"
                CREATE INDEX snippet_design_system IF NOT EXISTS
                FOR (s:Snippet)
                ON (s.design_system)
            "#,
            r#"
                CREATE INDEX snippet_created_at IF NOT EXISTS
                FOR (s:Snippet)
                ON (s.created_at)
            "#,
        ];

        for index_query in indexes {
            match self.graph.run(neo4rs::query(index_query)).await {
                Ok(_) => info!("Created index"),
                Err(e) => warn!("Index may already exist: {}", e),
            }
        }

        Ok(())
    }

    /// Create vector index for semantic similarity search
    /// Uses 1536 dimensions (OpenAI embedding size) with cosine distance
    async fn create_vector_index(&self) -> anyhow::Result<()> {
        // Neo4j 5.x vector index syntax
        let vector_index = r#"
            CREATE VECTOR INDEX ui_element_embedding IF NOT EXISTS
            FOR (e:UIElement)
            ON (e.embedding)
            OPTIONS {
                indexConfig: {
                    `vector.dimensions`: 1536,
                    `vector.similarity_function`: 'cosine'
                }
            }
        "#;

        match self.graph.run(neo4rs::query(vector_index)).await {
            Ok(_) => info!("Created vector index for embeddings (1536 dims, cosine)"),
            Err(e) => warn!("Vector index may already exist or not supported: {}", e),
        }

        Ok(())
    }

    /// Create fulltext index for semantic text search
    async fn create_fulltext_index(&self) -> anyhow::Result<()> {
        let fulltext_index = r#"
            CREATE FULLTEXT INDEX ui_element_fulltext IF NOT EXISTS
            FOR (e:UIElement)
            ON EACH [e.name, e.html_template, e.css_classes_text]
        "#;

        match self.graph.run(neo4rs::query(fulltext_index)).await {
            Ok(_) => info!("Created fulltext index for name, html_template, css_classes"),
            Err(e) => warn!("Fulltext index may already exist: {}", e),
        }

        Ok(())
    }

    /// Drop all custom indexes and constraints (for testing/reset)
    pub async fn drop_all(&self) -> anyhow::Result<()> {
        warn!("Dropping all custom indexes and constraints");

        let drops = vec![
            "DROP CONSTRAINT ui_element_id_unique IF EXISTS",
            "DROP CONSTRAINT design_system_name_unique IF EXISTS",
            "DROP CONSTRAINT snippet_id_unique IF EXISTS",
            "DROP INDEX ui_element_category IF EXISTS",
            "DROP INDEX ui_element_type IF EXISTS",
            "DROP INDEX ui_element_design_system IF EXISTS",
            "DROP INDEX ui_element_category_type IF EXISTS",
            "DROP INDEX ui_element_embedding IF EXISTS",
            "DROP INDEX ui_element_fulltext IF EXISTS",
            "DROP INDEX snippet_design_system IF EXISTS",
            "DROP INDEX snippet_created_at IF EXISTS",
        ];

        for drop_query in drops {
            let _ = self.graph.run(neo4rs::query(drop_query)).await;
        }

        Ok(())
    }
}
