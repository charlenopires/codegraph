//! Ontology Mapper - maps extracted entities to UI ontology

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::css::{CssStructure, TokenCategory};
use crate::design_system::{DesignSystemType, DetectionResult};
use crate::html::HtmlStructure;
use crate::javascript::JsStructure;

/// UI element category in the ontology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UICategory {
    // Layout
    Container,
    Grid,
    Flex,
    Stack,

    // Navigation
    Navigation,
    Menu,
    Breadcrumb,
    Tabs,
    Pagination,

    // Forms
    Form,
    Input,
    Select,
    Checkbox,
    Radio,
    Switch,
    Slider,
    DatePicker,

    // Actions
    Button,
    Link,
    IconButton,
    FAB,

    // Display
    Card,
    List,
    Table,
    Avatar,
    Badge,
    Chip,
    Tag,

    // Feedback
    Alert,
    Toast,
    Snackbar,
    Progress,
    Spinner,
    Skeleton,

    // Overlay
    Modal,
    Dialog,
    Drawer,
    Popover,
    Tooltip,
    ContextMenu,

    // Media
    Image,
    Video,
    Icon,

    // Typography
    Heading,
    Text,
    Label,

    // Other
    Divider,
    Spacer,
    Unknown,
}

impl UICategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Container => "container",
            Self::Grid => "grid",
            Self::Flex => "flex",
            Self::Stack => "stack",
            Self::Navigation => "navigation",
            Self::Menu => "menu",
            Self::ContextMenu => "context-menu",
            Self::Breadcrumb => "breadcrumb",
            Self::Tabs => "tabs",
            Self::Pagination => "pagination",
            Self::Form => "form",
            Self::Input => "input",
            Self::Select => "select",
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Switch => "switch",
            Self::Slider => "slider",
            Self::DatePicker => "date-picker",
            Self::Button => "button",
            Self::Link => "link",
            Self::IconButton => "icon-button",
            Self::FAB => "fab",
            Self::Card => "card",
            Self::List => "list",
            Self::Table => "table",
            Self::Avatar => "avatar",
            Self::Badge => "badge",
            Self::Chip => "chip",
            Self::Tag => "tag",
            Self::Alert => "alert",
            Self::Toast => "toast",
            Self::Snackbar => "snackbar",
            Self::Progress => "progress",
            Self::Spinner => "spinner",
            Self::Skeleton => "skeleton",
            Self::Modal => "modal",
            Self::Dialog => "dialog",
            Self::Drawer => "drawer",
            Self::Popover => "popover",
            Self::Tooltip => "tooltip",
            Self::Image => "image",
            Self::Video => "video",
            Self::Icon => "icon",
            Self::Heading => "heading",
            Self::Text => "text",
            Self::Label => "label",
            Self::Divider => "divider",
            Self::Spacer => "spacer",
            Self::Unknown => "unknown",
        }
    }
}

/// Mapped UI element in the ontology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappedElement {
    pub category: UICategory,
    pub element_type: String,
    pub design_system: Option<DesignSystemType>,
    pub classes: Vec<String>,
    pub properties: Vec<(String, String)>,
    pub children_categories: Vec<UICategory>,
    pub has_interactivity: bool,
}

/// Ontology mapping result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyMapping {
    pub elements: Vec<MappedElement>,
    pub categories_used: Vec<UICategory>,
    pub design_tokens: Vec<(String, String, TokenCategory)>,
    pub design_system: Option<DesignSystemType>,
}

/// Maps extracted code structures to UI ontology
pub struct OntologyMapper {
    tag_mappings: Vec<(&'static str, UICategory)>,
    class_mappings: Vec<(&'static str, UICategory)>,
}

impl Default for OntologyMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl OntologyMapper {
    pub fn new() -> Self {
        // Tag to category mappings
        let tag_mappings = vec![
            ("button", UICategory::Button),
            ("a", UICategory::Link),
            ("input", UICategory::Input),
            ("select", UICategory::Select),
            ("textarea", UICategory::Input),
            ("form", UICategory::Form),
            ("nav", UICategory::Navigation),
            ("ul", UICategory::List),
            ("ol", UICategory::List),
            ("table", UICategory::Table),
            ("img", UICategory::Image),
            ("video", UICategory::Video),
            ("h1", UICategory::Heading),
            ("h2", UICategory::Heading),
            ("h3", UICategory::Heading),
            ("h4", UICategory::Heading),
            ("h5", UICategory::Heading),
            ("h6", UICategory::Heading),
            ("p", UICategory::Text),
            ("span", UICategory::Text),
            ("label", UICategory::Label),
            ("hr", UICategory::Divider),
            ("dialog", UICategory::Dialog),
        ];

        // Class pattern to category mappings
        let class_mappings = vec![
            ("btn", UICategory::Button),
            ("button", UICategory::Button),
            ("card", UICategory::Card),
            ("modal", UICategory::Modal),
            ("dialog", UICategory::Dialog),
            ("drawer", UICategory::Drawer),
            ("nav", UICategory::Navigation),
            ("menu", UICategory::Menu),
            ("tab", UICategory::Tabs),
            ("alert", UICategory::Alert),
            ("toast", UICategory::Toast),
            ("badge", UICategory::Badge),
            ("chip", UICategory::Chip),
            ("tag", UICategory::Tag),
            ("avatar", UICategory::Avatar),
            ("tooltip", UICategory::Tooltip),
            ("popover", UICategory::Popover),
            ("progress", UICategory::Progress),
            ("spinner", UICategory::Spinner),
            ("skeleton", UICategory::Skeleton),
            ("container", UICategory::Container),
            ("grid", UICategory::Grid),
            ("flex", UICategory::Flex),
            ("stack", UICategory::Stack),
            ("form", UICategory::Form),
            ("input", UICategory::Input),
            ("select", UICategory::Select),
            ("checkbox", UICategory::Checkbox),
            ("radio", UICategory::Radio),
            ("switch", UICategory::Switch),
            ("slider", UICategory::Slider),
            ("icon", UICategory::Icon),
            ("list", UICategory::List),
            ("table", UICategory::Table),
            ("breadcrumb", UICategory::Breadcrumb),
            ("pagination", UICategory::Pagination),
        ];

        Self {
            tag_mappings,
            class_mappings,
        }
    }

    /// Map HTML structure to ontology
    pub fn map_html(&self, html: &HtmlStructure, ds_result: &DetectionResult) -> OntologyMapping {
        let mut elements = Vec::new();
        let mut categories_used = HashSet::new();

        for element in &html.elements {
            let mapped = self.map_element(
                &element.tag,
                &element.classes,
                &element.attributes,
                ds_result.design_system,
            );
            categories_used.insert(mapped.category);
            elements.push(mapped);
        }

        OntologyMapping {
            elements,
            categories_used: categories_used.into_iter().collect(),
            design_tokens: Vec::new(),
            design_system: Some(ds_result.design_system),
        }
    }

    /// Map CSS structure to ontology (for design tokens)
    pub fn map_css(&self, css: &CssStructure) -> Vec<(String, String, TokenCategory)> {
        css.design_tokens
            .iter()
            .map(|token| (token.name.clone(), token.value.clone(), token.category))
            .collect()
    }

    /// Map JavaScript structure to detect interactivity
    pub fn has_interactivity(&self, js: &JsStructure) -> bool {
        !js.event_handlers.is_empty() || !js.dom_calls.is_empty()
    }

    /// Map a single element to ontology category
    fn map_element(
        &self,
        tag: &str,
        classes: &[String],
        attributes: &[(String, String)],
        design_system: DesignSystemType,
    ) -> MappedElement {
        // First, check tag mapping
        let mut category = UICategory::Unknown;

        for (t, cat) in &self.tag_mappings {
            if tag == *t {
                category = *cat;
                break;
            }
        }

        // Then, check class mappings (may override tag)
        let classes_lower: Vec<String> = classes.iter().map(|c| c.to_lowercase()).collect();
        let all_classes = classes_lower.join(" ");

        for (pattern, cat) in &self.class_mappings {
            if all_classes.contains(pattern) {
                category = *cat;
                break;
            }
        }

        // Check for input types
        if tag == "input" {
            for (name, value) in attributes {
                if name == "type" {
                    category = match value.as_str() {
                        "checkbox" => UICategory::Checkbox,
                        "radio" => UICategory::Radio,
                        "range" => UICategory::Slider,
                        "date" | "datetime-local" => UICategory::DatePicker,
                        "submit" | "button" => UICategory::Button,
                        _ => UICategory::Input,
                    };
                    break;
                }
            }
        }

        // Check for role attribute (ARIA)
        for (name, value) in attributes {
            if name == "role" {
                category = match value.as_str() {
                    "button" => UICategory::Button,
                    "navigation" => UICategory::Navigation,
                    "dialog" => UICategory::Dialog,
                    "menu" => UICategory::Menu,
                    "menuitem" => UICategory::Menu,
                    "tab" => UICategory::Tabs,
                    "tablist" => UICategory::Tabs,
                    "alert" => UICategory::Alert,
                    "tooltip" => UICategory::Tooltip,
                    "progressbar" => UICategory::Progress,
                    "checkbox" => UICategory::Checkbox,
                    "radio" => UICategory::Radio,
                    "switch" => UICategory::Switch,
                    "slider" => UICategory::Slider,
                    "img" => UICategory::Image,
                    "link" => UICategory::Link,
                    "list" => UICategory::List,
                    "listitem" => UICategory::List,
                    "table" => UICategory::Table,
                    _ => category,
                };
                break;
            }
        }

        let element_type = format!("{}-{}", design_system.as_str(), category.as_str());

        MappedElement {
            category,
            element_type,
            design_system: Some(design_system),
            classes: classes.to_vec(),
            properties: attributes.to_vec(),
            children_categories: Vec::new(),
            has_interactivity: false,
        }
    }

    /// Full mapping combining all structures
    pub fn map_full(
        &self,
        html: &HtmlStructure,
        css: &CssStructure,
        js: &JsStructure,
        ds_result: &DetectionResult,
    ) -> OntologyMapping {
        let mut mapping = self.map_html(html, ds_result);
        mapping.design_tokens = self.map_css(css);

        let has_interactive = self.has_interactivity(js);
        for elem in &mut mapping.elements {
            elem.has_interactivity = has_interactive;
        }

        mapping
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design_system::DesignSystemDetector;
    use crate::html::HtmlParser;

    #[test]
    fn test_map_button() {
        let mapper = OntologyMapper::new();
        let detector = DesignSystemDetector::new();

        let mut html_parser = HtmlParser::new();
        let html = html_parser.parse(r#"<button class="btn btn-primary">Click me</button>"#).unwrap();
        let ds_result = detector.detect_from_classes(&html.classes);

        let mapping = mapper.map_html(&html, &ds_result);

        assert!(!mapping.elements.is_empty());
        assert_eq!(mapping.elements[0].category, UICategory::Button);
    }
}
