//! Collection definitions for design system embeddings
//!
//! Each design system has its own Qdrant collection to enable
//! efficient filtering and targeted searches.

use serde::{Deserialize, Serialize};

/// Design system collection names
pub const COLLECTION_MATERIAL: &str = "ui_material";
pub const COLLECTION_TAILWIND: &str = "ui_tailwind";
pub const COLLECTION_CHAKRA: &str = "ui_chakra";
pub const COLLECTION_BOOTSTRAP: &str = "ui_bootstrap";
pub const COLLECTION_CUSTOM: &str = "ui_custom";

/// All available collections
pub const COLLECTIONS: &[&str] = &[
    COLLECTION_MATERIAL,
    COLLECTION_TAILWIND,
    COLLECTION_CHAKRA,
    COLLECTION_BOOTSTRAP,
    COLLECTION_CUSTOM,
];

/// Represents a Qdrant collection for a design system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Collection name in Qdrant
    pub name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Description of the design system
    pub description: String,
}

impl Collection {
    /// Create a new collection definition
    pub fn new(name: impl Into<String>, display_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: display_name.into(),
            description: description.into(),
        }
    }

    /// Get all predefined collections
    pub fn all() -> Vec<Collection> {
        vec![
            Collection::new(
                COLLECTION_MATERIAL,
                "Material UI",
                "Google's Material Design components and patterns",
            ),
            Collection::new(
                COLLECTION_TAILWIND,
                "Tailwind CSS",
                "Utility-first CSS framework components",
            ),
            Collection::new(
                COLLECTION_CHAKRA,
                "Chakra UI",
                "Simple, modular and accessible component library",
            ),
            Collection::new(
                COLLECTION_BOOTSTRAP,
                "Bootstrap",
                "Popular CSS framework for responsive layouts",
            ),
            Collection::new(
                COLLECTION_CUSTOM,
                "Custom",
                "Custom and framework-agnostic components",
            ),
        ]
    }

    /// Get collection by design system name
    pub fn from_design_system(design_system: &str) -> Option<Collection> {
        let name = design_system_to_collection(design_system)?;
        Collection::all().into_iter().find(|c| c.name == name)
    }
}

/// Map design system name to collection name
pub fn design_system_to_collection(design_system: &str) -> Option<&'static str> {
    match design_system.to_lowercase().as_str() {
        "material" | "material-ui" | "mui" => Some(COLLECTION_MATERIAL),
        "tailwind" | "tailwindcss" | "tailwind-css" => Some(COLLECTION_TAILWIND),
        "chakra" | "chakra-ui" => Some(COLLECTION_CHAKRA),
        "bootstrap" => Some(COLLECTION_BOOTSTRAP),
        "custom" | "" => Some(COLLECTION_CUSTOM),
        _ => None,
    }
}

/// Map collection name back to canonical design system name
pub fn collection_to_design_system(collection: &str) -> Option<&'static str> {
    match collection {
        COLLECTION_MATERIAL => Some("material-ui"),
        COLLECTION_TAILWIND => Some("tailwind"),
        COLLECTION_CHAKRA => Some("chakra"),
        COLLECTION_BOOTSTRAP => Some("bootstrap"),
        COLLECTION_CUSTOM => Some("custom"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collections_count() {
        assert_eq!(COLLECTIONS.len(), 5);
        assert_eq!(Collection::all().len(), 5);
    }

    #[test]
    fn test_design_system_to_collection() {
        assert_eq!(design_system_to_collection("tailwind"), Some(COLLECTION_TAILWIND));
        assert_eq!(design_system_to_collection("TailwindCSS"), Some(COLLECTION_TAILWIND));
        assert_eq!(design_system_to_collection("material-ui"), Some(COLLECTION_MATERIAL));
        assert_eq!(design_system_to_collection("mui"), Some(COLLECTION_MATERIAL));
        assert_eq!(design_system_to_collection("chakra-ui"), Some(COLLECTION_CHAKRA));
        assert_eq!(design_system_to_collection("bootstrap"), Some(COLLECTION_BOOTSTRAP));
        assert_eq!(design_system_to_collection("custom"), Some(COLLECTION_CUSTOM));
        assert_eq!(design_system_to_collection(""), Some(COLLECTION_CUSTOM));
        assert_eq!(design_system_to_collection("unknown"), None);
    }

    #[test]
    fn test_collection_to_design_system() {
        assert_eq!(collection_to_design_system(COLLECTION_TAILWIND), Some("tailwind"));
        assert_eq!(collection_to_design_system(COLLECTION_MATERIAL), Some("material-ui"));
        assert_eq!(collection_to_design_system("unknown"), None);
    }

    #[test]
    fn test_from_design_system() {
        let collection = Collection::from_design_system("tailwind").unwrap();
        assert_eq!(collection.name, COLLECTION_TAILWIND);
        assert_eq!(collection.display_name, "Tailwind CSS");

        assert!(Collection::from_design_system("unknown").is_none());
    }
}
