//! Graph entities - Node types for the UI component graph

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A UI element node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    /// Unique identifier
    pub id: Uuid,
    /// Element name (e.g., "PrimaryButton", "LoginForm")
    pub name: String,
    /// Category (e.g., "button", "form", "card")
    pub category: String,
    /// Element type (e.g., "atomic", "composite")
    pub element_type: String,
    /// Design system this belongs to
    pub design_system: Option<String>,
    /// HTML template
    pub html_template: Option<String>,
    /// CSS classes used
    pub css_classes: Vec<String>,
    /// Tags/attributes
    pub tags: Vec<String>,
    /// Embedding vector (1536 dims for OpenAI)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl UIElement {
    pub fn new(name: impl Into<String>, category: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            category: category.into(),
            element_type: "atomic".to_string(),
            design_system: None,
            html_template: None,
            css_classes: vec![],
            tags: vec![],
            embedding: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn with_element_type(mut self, element_type: impl Into<String>) -> Self {
        self.element_type = element_type.into();
        self
    }

    pub fn with_design_system(mut self, design_system: impl Into<String>) -> Self {
        self.design_system = Some(design_system.into());
        self
    }

    pub fn with_html_template(mut self, template: impl Into<String>) -> Self {
        self.html_template = Some(template.into());
        self
    }

    pub fn with_css_classes(mut self, classes: Vec<String>) -> Self {
        self.css_classes = classes;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

/// A design system node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSystem {
    /// Unique name (e.g., "tailwind", "bootstrap", "material")
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Version
    pub version: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Base URL for documentation
    pub docs_url: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl DesignSystem {
    pub fn new(name: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: display_name.into(),
            version: None,
            description: None,
            docs_url: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarElement {
    pub element: UIElement,
    pub similarity: f32,
}

/// A code snippet containing one or more UI elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    /// Unique identifier
    pub id: Uuid,
    /// Optional name for the snippet
    pub name: Option<String>,
    /// Original HTML content
    pub html: String,
    /// Original CSS content (if provided)
    pub css: Option<String>,
    /// Original JavaScript content (if provided)
    pub js: Option<String>,
    /// Detected design system
    pub design_system: Option<String>,
    /// User-provided tags
    pub tags: Vec<String>,
    /// IDs of UI elements extracted from this snippet
    pub element_ids: Vec<Uuid>,
    /// Number of elements in this snippet
    pub element_count: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Snippet {
    pub fn new(html: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: None,
            html: html.into(),
            css: None,
            js: None,
            design_system: None,
            tags: vec![],
            element_ids: vec![],
            element_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_css(mut self, css: impl Into<String>) -> Self {
        self.css = Some(css.into());
        self
    }

    pub fn with_js(mut self, js: impl Into<String>) -> Self {
        self.js = Some(js.into());
        self
    }

    pub fn with_design_system(mut self, design_system: impl Into<String>) -> Self {
        self.design_system = Some(design_system.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_element_ids(mut self, element_ids: Vec<Uuid>) -> Self {
        self.element_count = element_ids.len() as u32;
        self.element_ids = element_ids;
        self
    }
}

/// Snippet summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetSummary {
    pub id: Uuid,
    pub name: Option<String>,
    pub design_system: Option<String>,
    pub element_count: u32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}
