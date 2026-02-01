//! Relation management - Graph relationships between UI elements

use neo4rs::{query, Graph};
use tracing::{debug, info};
use uuid::Uuid;

/// Relationship types in the UI component graph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType {
    /// Parent-child containment
    HasChild,
    /// Element has a style
    HasStyle,
    /// Element has a state (hover, active, disabled)
    HasState,
    /// Element triggers an event
    TriggersEvent,
    /// Element belongs to a design system
    BelongsToDesignSystem,
    /// Elements are semantically similar
    SimilarTo,
    /// Element can replace another
    CanReplace,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HasChild => "HAS_CHILD",
            Self::HasStyle => "HAS_STYLE",
            Self::HasState => "HAS_STATE",
            Self::TriggersEvent => "TRIGGERS_EVENT",
            Self::BelongsToDesignSystem => "BELONGS_TO_DESIGN_SYSTEM",
            Self::SimilarTo => "SIMILAR_TO",
            Self::CanReplace => "CAN_REPLACE",
        }
    }
}

/// Manages relationships between graph nodes
pub struct RelationManager {
    graph: Graph,
}

impl RelationManager {
    pub fn new(graph: Graph) -> Self {
        Self { graph }
    }

    /// Create a relationship between two UIElements
    pub async fn create_relation(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        relation_type: RelationType,
    ) -> anyhow::Result<()> {
        self.create_relation_with_props(from_id, to_id, relation_type, None)
            .await
    }

    /// Create a relationship with optional properties
    pub async fn create_relation_with_props(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        relation_type: RelationType,
        properties: Option<serde_json::Value>,
    ) -> anyhow::Result<()> {
        let rel_type = relation_type.as_str();

        let cypher = format!(
            r#"
            MATCH (a:UIElement {{id: $from_id}})
            MATCH (b:UIElement {{id: $to_id}})
            MERGE (a)-[r:{}]->(b)
            SET r.created_at = datetime()
            SET r += $props
            RETURN r
            "#,
            rel_type
        );

        let props = properties.unwrap_or(serde_json::json!({}));

        self.graph
            .run(
                query(&cypher)
                    .param("from_id", from_id.to_string())
                    .param("to_id", to_id.to_string())
                    .param("props", props.to_string()),
            )
            .await?;

        debug!(
            "Created {} relation: {} -> {}",
            rel_type, from_id, to_id
        );
        Ok(())
    }

    /// Link UIElement to DesignSystem
    pub async fn link_to_design_system(
        &self,
        element_id: Uuid,
        design_system_name: &str,
    ) -> anyhow::Result<()> {
        let cypher = r#"
            MATCH (e:UIElement {id: $element_id})
            MATCH (d:DesignSystem {name: $ds_name})
            MERGE (e)-[r:BELONGS_TO_DESIGN_SYSTEM]->(d)
            SET r.created_at = datetime()
            RETURN r
        "#;

        self.graph
            .run(
                query(cypher)
                    .param("element_id", element_id.to_string())
                    .param("ds_name", design_system_name),
            )
            .await?;

        info!(
            "Linked element {} to design system {}",
            element_id, design_system_name
        );
        Ok(())
    }

    /// Create similarity relationship with score
    pub async fn create_similarity(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        similarity_score: f32,
    ) -> anyhow::Result<()> {
        let cypher = r#"
            MATCH (a:UIElement {id: $from_id})
            MATCH (b:UIElement {id: $to_id})
            MERGE (a)-[r:SIMILAR_TO]->(b)
            SET r.score = $score
            SET r.created_at = datetime()
            RETURN r
        "#;

        self.graph
            .run(
                query(cypher)
                    .param("from_id", from_id.to_string())
                    .param("to_id", to_id.to_string())
                    .param("score", similarity_score),
            )
            .await?;

        debug!(
            "Created SIMILAR_TO relation: {} -> {} (score: {})",
            from_id, to_id, similarity_score
        );
        Ok(())
    }

    /// Delete a specific relationship
    pub async fn delete_relation(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        relation_type: RelationType,
    ) -> anyhow::Result<()> {
        let rel_type = relation_type.as_str();

        let cypher = format!(
            r#"
            MATCH (a:UIElement {{id: $from_id}})-[r:{}]->(b:UIElement {{id: $to_id}})
            DELETE r
            "#,
            rel_type
        );

        self.graph
            .run(
                query(&cypher)
                    .param("from_id", from_id.to_string())
                    .param("to_id", to_id.to_string()),
            )
            .await?;

        Ok(())
    }

    /// Get all children of an element
    pub async fn get_children(&self, parent_id: Uuid) -> anyhow::Result<Vec<Uuid>> {
        let cypher = r#"
            MATCH (p:UIElement {id: $parent_id})-[:HAS_CHILD]->(c:UIElement)
            RETURN c.id as child_id
        "#;

        let mut result = self
            .graph
            .execute(query(cypher).param("parent_id", parent_id.to_string()))
            .await?;

        let mut children = Vec::new();
        while let Some(row) = result.next().await? {
            if let Ok(id_str) = row.get::<String>("child_id") {
                if let Ok(id) = Uuid::parse_str(&id_str) {
                    children.push(id);
                }
            }
        }

        Ok(children)
    }

    /// Get similar elements by relationship
    pub async fn get_similar(&self, element_id: Uuid, min_score: f32) -> anyhow::Result<Vec<(Uuid, f32)>> {
        let cypher = r#"
            MATCH (e:UIElement {id: $element_id})-[r:SIMILAR_TO]->(s:UIElement)
            WHERE r.score >= $min_score
            RETURN s.id as similar_id, r.score as score
            ORDER BY r.score DESC
        "#;

        let mut result = self
            .graph
            .execute(
                query(cypher)
                    .param("element_id", element_id.to_string())
                    .param("min_score", min_score),
            )
            .await?;

        let mut similar = Vec::new();
        while let Some(row) = result.next().await? {
            if let (Ok(id_str), Ok(score)) = (
                row.get::<String>("similar_id"),
                row.get::<f64>("score"),
            ) {
                if let Ok(id) = Uuid::parse_str(&id_str) {
                    similar.push((id, score as f32));
                }
            }
        }

        Ok(similar)
    }

    /// Get the graph degree (number of relationships) for an element
    pub async fn get_degree(&self, element_id: Uuid) -> anyhow::Result<u32> {
        let cypher = r#"
            MATCH (e:UIElement {id: $element_id})-[r]-()
            RETURN count(r) as degree
        "#;

        let mut result = self
            .graph
            .execute(query(cypher).param("element_id", element_id.to_string()))
            .await?;

        if let Some(row) = result.next().await? {
            let degree: i64 = row.get("degree").unwrap_or(0);
            return Ok(degree as u32);
        }

        Ok(0)
    }
}
