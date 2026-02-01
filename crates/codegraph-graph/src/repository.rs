//! Neo4j Repository - CRUD operations for UI elements
//!
//! Optimized for <100ms latency on simple queries and 10k+ element support.

use std::env;
use std::time::Instant;

use neo4rs::{query, ConfigBuilder, Graph};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::entities::{DesignSystem, SimilarElement, Snippet, SnippetSummary, UIElement};
use crate::relations::RelationManager;
use crate::schema::SchemaManager;

/// Neo4j repository for UI element persistence
pub struct Neo4jRepository {
    graph: Graph,
    schema: SchemaManager,
    relations: RelationManager,
    latency_target_ms: u64,
}

impl Neo4jRepository {
    /// Create a new repository from environment variables
    pub async fn new() -> anyhow::Result<Self> {
        let uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
        let user = env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
        let password = env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "codegraph123".to_string());

        Self::connect(&uri, &user, &password).await
    }

    /// Connect to Neo4j with explicit credentials
    pub async fn connect(uri: &str, user: &str, password: &str) -> anyhow::Result<Self> {
        info!("Connecting to Neo4j at {}", uri);

        let config = ConfigBuilder::default()
            .uri(uri)
            .user(user)
            .password(password)
            .max_connections(50) // Support high concurrency for 10k+ elements
            .build()?;

        let graph = Graph::connect(config).await?;

        let schema = SchemaManager::new(graph.clone());
        let relations = RelationManager::new(graph.clone());

        Ok(Self {
            graph,
            schema,
            relations,
            latency_target_ms: 100,
        })
    }

    /// Initialize schema (constraints + indexes)
    pub async fn initialize_schema(&self) -> anyhow::Result<()> {
        self.schema.initialize().await
    }

    /// Get schema manager
    pub fn schema(&self) -> &SchemaManager {
        &self.schema
    }

    /// Get relation manager
    pub fn relations(&self) -> &RelationManager {
        &self.relations
    }

    // ==================== CRUD Operations ====================

    /// Save a UIElement (create or update)
    pub async fn save(&self, element: &UIElement) -> anyhow::Result<()> {
        let start = Instant::now();

        let cypher = r#"
            MERGE (e:UIElement {id: $id})
            SET e.name = $name,
                e.category = $category,
                e.element_type = $element_type,
                e.design_system = $design_system,
                e.html_template = $html_template,
                e.css_classes = $css_classes,
                e.css_classes_text = $css_classes_text,
                e.tags = $tags,
                e.embedding = $embedding,
                e.created_at = coalesce(e.created_at, datetime()),
                e.updated_at = datetime()
            RETURN e
        "#;

        let css_classes_text = element.css_classes.join(" ");

        self.graph
            .run(
                query(cypher)
                    .param("id", element.id.to_string())
                    .param("name", element.name.clone())
                    .param("category", element.category.clone())
                    .param("element_type", element.element_type.clone())
                    .param("design_system", element.design_system.clone())
                    .param("html_template", element.html_template.clone())
                    .param("css_classes", element.css_classes.clone())
                    .param("css_classes_text", css_classes_text)
                    .param("tags", element.tags.clone())
                    .param("embedding", element.embedding.clone()),
            )
            .await?;

        self.check_latency("save", start);
        Ok(())
    }

    /// Find element by ID
    pub async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<UIElement>> {
        let start = Instant::now();

        let cypher = r#"
            MATCH (e:UIElement {id: $id})
            RETURN e
        "#;

        let mut result = self
            .graph
            .execute(query(cypher).param("id", id.to_string()))
            .await?;

        let element = if let Some(row) = result.next().await? {
            Some(self.row_to_element(&row)?)
        } else {
            None
        };

        self.check_latency("find_by_id", start);
        Ok(element)
    }

    /// Find elements by category
    pub async fn find_by_category(&self, category: &str) -> anyhow::Result<Vec<UIElement>> {
        let start = Instant::now();

        let cypher = r#"
            MATCH (e:UIElement {category: $category})
            RETURN e
            ORDER BY e.name
        "#;

        let mut result = self
            .graph
            .execute(query(cypher).param("category", category))
            .await?;

        let mut elements = Vec::new();
        while let Some(row) = result.next().await? {
            elements.push(self.row_to_element(&row)?);
        }

        self.check_latency("find_by_category", start);
        Ok(elements)
    }

    /// Find similar elements using vector similarity
    pub async fn find_similar(
        &self,
        embedding: &[f32],
        limit: usize,
    ) -> anyhow::Result<Vec<SimilarElement>> {
        let start = Instant::now();

        // Use Neo4j vector index for similarity search
        let cypher = r#"
            CALL db.index.vector.queryNodes('ui_element_embedding', $limit, $embedding)
            YIELD node, score
            RETURN node as e, score
            ORDER BY score DESC
        "#;

        let mut result = self
            .graph
            .execute(
                query(cypher)
                    .param("limit", limit as i64)
                    .param("embedding", embedding.to_vec()),
            )
            .await?;

        let mut elements = Vec::new();
        while let Some(row) = result.next().await? {
            let element = self.row_to_element(&row)?;
            let score: f64 = row.get("score").unwrap_or(0.0);
            elements.push(SimilarElement {
                element,
                similarity: score as f32,
            });
        }

        self.check_latency("find_similar", start);
        Ok(elements)
    }

    /// Fulltext search across name, html_template, css_classes
    pub async fn fulltext_search(
        &self,
        search_term: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<SimilarElement>> {
        let start = Instant::now();

        let cypher = r#"
            CALL db.index.fulltext.queryNodes('ui_element_fulltext', $search_term)
            YIELD node, score
            RETURN node as e, score
            ORDER BY score DESC
            LIMIT $limit
        "#;

        let mut result = self
            .graph
            .execute(
                query(cypher)
                    .param("search_term", search_term)
                    .param("limit", limit as i64),
            )
            .await?;

        let mut elements = Vec::new();
        while let Some(row) = result.next().await? {
            let element = self.row_to_element(&row)?;
            let score: f64 = row.get("score").unwrap_or(0.0);
            elements.push(SimilarElement {
                element,
                similarity: score as f32,
            });
        }

        self.check_latency("fulltext_search", start);
        Ok(elements)
    }

    /// Delete element by ID
    pub async fn delete(&self, id: Uuid) -> anyhow::Result<bool> {
        let start = Instant::now();

        let cypher = r#"
            MATCH (e:UIElement {id: $id})
            DETACH DELETE e
            RETURN count(e) as deleted
        "#;

        let mut result = self
            .graph
            .execute(query(cypher).param("id", id.to_string()))
            .await?;

        let deleted = if let Some(row) = result.next().await? {
            let count: i64 = row.get("deleted").unwrap_or(0);
            count > 0
        } else {
            false
        };

        self.check_latency("delete", start);
        Ok(deleted)
    }

    // ==================== Design System Operations ====================

    /// Save a design system
    pub async fn save_design_system(&self, ds: &DesignSystem) -> anyhow::Result<()> {
        let cypher = r#"
            MERGE (d:DesignSystem {name: $name})
            SET d.display_name = $display_name,
                d.version = $version,
                d.description = $description,
                d.docs_url = $docs_url,
                d.created_at = coalesce(d.created_at, datetime())
            RETURN d
        "#;

        self.graph
            .run(
                query(cypher)
                    .param("name", ds.name.clone())
                    .param("display_name", ds.display_name.clone())
                    .param("version", ds.version.clone())
                    .param("description", ds.description.clone())
                    .param("docs_url", ds.docs_url.clone()),
            )
            .await?;

        Ok(())
    }

    // ==================== Snippet Operations ====================

    /// Save a snippet and create HAS_ELEMENT relationships
    pub async fn save_snippet(&self, snippet: &Snippet) -> anyhow::Result<()> {
        let start = Instant::now();

        // Create the Snippet node
        let cypher = r#"
            MERGE (s:Snippet {id: $id})
            SET s.name = $name,
                s.html = $html,
                s.css = $css,
                s.js = $js,
                s.design_system = $design_system,
                s.tags = $tags,
                s.element_count = $element_count,
                s.created_at = coalesce(s.created_at, datetime()),
                s.updated_at = datetime()
            RETURN s
        "#;

        self.graph
            .run(
                query(cypher)
                    .param("id", snippet.id.to_string())
                    .param("name", snippet.name.clone())
                    .param("html", snippet.html.clone())
                    .param("css", snippet.css.clone())
                    .param("js", snippet.js.clone())
                    .param("design_system", snippet.design_system.clone())
                    .param("tags", snippet.tags.clone())
                    .param("element_count", snippet.element_count as i64),
            )
            .await?;

        // Create HAS_ELEMENT relationships
        for element_id in &snippet.element_ids {
            let rel_cypher = r#"
                MATCH (s:Snippet {id: $snippet_id})
                MATCH (e:UIElement {id: $element_id})
                MERGE (s)-[:HAS_ELEMENT]->(e)
            "#;

            self.graph
                .run(
                    query(rel_cypher)
                        .param("snippet_id", snippet.id.to_string())
                        .param("element_id", element_id.to_string()),
                )
                .await?;
        }

        self.check_latency("save_snippet", start);
        Ok(())
    }

    /// Find snippet by ID with its elements
    pub async fn find_snippet_by_id(&self, id: Uuid) -> anyhow::Result<Option<Snippet>> {
        let start = Instant::now();

        let cypher = r#"
            MATCH (s:Snippet {id: $id})
            OPTIONAL MATCH (s)-[:HAS_ELEMENT]->(e:UIElement)
            WITH s, collect(e.id) as element_ids
            RETURN s, element_ids
        "#;

        let mut result = self
            .graph
            .execute(query(cypher).param("id", id.to_string()))
            .await?;

        let snippet = if let Some(row) = result.next().await? {
            Some(self.row_to_snippet(&row)?)
        } else {
            None
        };

        self.check_latency("find_snippet_by_id", start);
        Ok(snippet)
    }

    /// List snippets with pagination
    pub async fn list_snippets(
        &self,
        page: u32,
        per_page: u32,
        design_system: Option<&str>,
        category: Option<&str>,
    ) -> anyhow::Result<(Vec<SnippetSummary>, u64)> {
        let start = Instant::now();
        let offset = (page.saturating_sub(1)) * per_page;

        // Build dynamic WHERE clause
        let mut where_clauses = Vec::new();
        if design_system.is_some() {
            where_clauses.push("s.design_system = $design_system");
        }
        if category.is_some() {
            where_clauses.push("EXISTS { (s)-[:HAS_ELEMENT]->(e:UIElement) WHERE e.category = $category }");
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Count total
        let count_cypher = format!(
            "MATCH (s:Snippet) {} RETURN count(s) as total",
            where_clause
        );

        let mut count_query = query(&count_cypher);
        if let Some(ds) = design_system {
            count_query = count_query.param("design_system", ds);
        }
        if let Some(cat) = category {
            count_query = count_query.param("category", cat);
        }

        let mut count_result = self.graph.execute(count_query).await?;
        let total: u64 = if let Some(row) = count_result.next().await? {
            let count: i64 = row.get("total").unwrap_or(0);
            count as u64
        } else {
            0
        };

        // Fetch page
        let list_cypher = format!(
            r#"
            MATCH (s:Snippet)
            {}
            OPTIONAL MATCH (s)-[:HAS_ELEMENT]->(e:UIElement)
            WITH s, count(e) as element_count
            RETURN s, element_count
            ORDER BY s.created_at DESC
            SKIP $offset
            LIMIT $limit
            "#,
            where_clause
        );

        let mut list_query = query(&list_cypher)
            .param("offset", offset as i64)
            .param("limit", per_page as i64);

        if let Some(ds) = design_system {
            list_query = list_query.param("design_system", ds);
        }
        if let Some(cat) = category {
            list_query = list_query.param("category", cat);
        }

        let mut result = self.graph.execute(list_query).await?;
        let mut snippets = Vec::new();

        while let Some(row) = result.next().await? {
            snippets.push(self.row_to_snippet_summary(&row)?);
        }

        self.check_latency("list_snippets", start);
        Ok((snippets, total))
    }

    /// Delete snippet and optionally orphaned elements
    pub async fn delete_snippet(&self, id: Uuid, delete_orphans: bool) -> anyhow::Result<bool> {
        let start = Instant::now();

        if delete_orphans {
            // First, find elements that will be orphaned
            let orphan_cypher = r#"
                MATCH (s:Snippet {id: $id})-[:HAS_ELEMENT]->(e:UIElement)
                WHERE NOT EXISTS {
                    (other:Snippet)-[:HAS_ELEMENT]->(e)
                    WHERE other.id <> $id
                }
                DETACH DELETE e
            "#;

            self.graph
                .run(query(orphan_cypher).param("id", id.to_string()))
                .await?;
        }

        // Delete the snippet
        let cypher = r#"
            MATCH (s:Snippet {id: $id})
            DETACH DELETE s
            RETURN count(s) as deleted
        "#;

        let mut result = self
            .graph
            .execute(query(cypher).param("id", id.to_string()))
            .await?;

        let deleted = if let Some(row) = result.next().await? {
            let count: i64 = row.get("deleted").unwrap_or(0);
            count > 0
        } else {
            false
        };

        self.check_latency("delete_snippet", start);
        Ok(deleted)
    }

    /// Count snippets
    pub async fn count_snippets(&self) -> anyhow::Result<u64> {
        let cypher = "MATCH (s:Snippet) RETURN count(s) as count";
        let mut result = self.graph.execute(query(cypher)).await?;

        if let Some(row) = result.next().await? {
            let count: i64 = row.get("count").unwrap_or(0);
            return Ok(count as u64);
        }

        Ok(0)
    }

    // ==================== Utility Methods ====================

    /// Get total element count
    pub async fn count(&self) -> anyhow::Result<u64> {
        let cypher = "MATCH (e:UIElement) RETURN count(e) as count";
        let mut result = self.graph.execute(query(cypher)).await?;

        if let Some(row) = result.next().await? {
            let count: i64 = row.get("count").unwrap_or(0);
            return Ok(count as u64);
        }

        Ok(0)
    }

    /// Convert Neo4j row to UIElement
    fn row_to_element(&self, row: &neo4rs::Row) -> anyhow::Result<UIElement> {
        let node: neo4rs::Node = row.get("e")?;

        let id_str: String = node.get("id")?;
        let id = Uuid::parse_str(&id_str)?;

        Ok(UIElement {
            id,
            name: node.get("name").unwrap_or_default(),
            category: node.get("category").unwrap_or_default(),
            element_type: node.get("element_type").unwrap_or_default(),
            design_system: node.get("design_system").ok(),
            html_template: node.get("html_template").ok(),
            css_classes: node.get("css_classes").unwrap_or_default(),
            tags: node.get("tags").unwrap_or_default(),
            embedding: node.get("embedding").ok(),
            created_at: chrono::Utc::now(), // TODO: parse from node
            updated_at: chrono::Utc::now(),
        })
    }

    /// Convert Neo4j row to Snippet
    fn row_to_snippet(&self, row: &neo4rs::Row) -> anyhow::Result<Snippet> {
        let node: neo4rs::Node = row.get("s")?;

        let id_str: String = node.get("id")?;
        let id = Uuid::parse_str(&id_str)?;

        // Parse element IDs from the collected list
        let element_id_strs: Vec<String> = row.get("element_ids").unwrap_or_default();
        let element_ids: Vec<Uuid> = element_id_strs
            .iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect();

        Ok(Snippet {
            id,
            name: node.get("name").ok(),
            html: node.get("html").unwrap_or_default(),
            css: node.get("css").ok(),
            js: node.get("js").ok(),
            design_system: node.get("design_system").ok(),
            tags: node.get("tags").unwrap_or_default(),
            element_ids: element_ids.clone(),
            element_count: element_ids.len() as u32,
            created_at: chrono::Utc::now(), // TODO: parse from node
            updated_at: chrono::Utc::now(),
        })
    }

    /// Convert Neo4j row to SnippetSummary
    fn row_to_snippet_summary(&self, row: &neo4rs::Row) -> anyhow::Result<SnippetSummary> {
        let node: neo4rs::Node = row.get("s")?;

        let id_str: String = node.get("id")?;
        let id = Uuid::parse_str(&id_str)?;

        let element_count: i64 = row.get("element_count").unwrap_or(0);

        Ok(SnippetSummary {
            id,
            name: node.get("name").ok(),
            design_system: node.get("design_system").ok(),
            element_count: element_count as u32,
            tags: node.get("tags").unwrap_or_default(),
            created_at: chrono::Utc::now(), // TODO: parse from node
        })
    }

    /// Check query latency against target
    fn check_latency(&self, operation: &str, start: Instant) {
        let elapsed_ms = start.elapsed().as_millis() as u64;
        if elapsed_ms > self.latency_target_ms {
            warn!(
                "{} latency {}ms exceeded target {}ms",
                operation, elapsed_ms, self.latency_target_ms
            );
        } else {
            debug!("{} completed in {}ms", operation, elapsed_ms);
        }
    }
}
