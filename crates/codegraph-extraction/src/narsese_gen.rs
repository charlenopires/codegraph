//! Narsese Generator - generates NARS statements from UI elements

use serde::{Deserialize, Serialize};

use crate::design_system::DesignSystemType;
use crate::ontology::{MappedElement, OntologyMapping, UICategory};

/// Truth value for NARS statements
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TruthValue {
    /// Frequency (0.0 - 1.0)
    pub frequency: f32,
    /// Confidence (0.0 - 1.0)
    pub confidence: f32,
}

impl TruthValue {
    pub fn new(frequency: f32, confidence: f32) -> Self {
        Self {
            frequency: frequency.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Default truth value for extracted facts
    pub fn extracted() -> Self {
        Self::new(0.9, 0.5)
    }

    /// High confidence truth value
    pub fn certain() -> Self {
        Self::new(1.0, 0.9)
    }

    /// Format as NARS truth value string
    pub fn to_narsese(&self) -> String {
        format!("{{{:.2} {:.2}}}", self.frequency, self.confidence)
    }
}

impl Default for TruthValue {
    fn default() -> Self {
        Self::extracted()
    }
}

/// Generated Narsese statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarseseStatement {
    pub statement: String,
    pub truth_value: TruthValue,
    pub statement_type: StatementType,
}

/// Type of Narsese statement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatementType {
    /// Inheritance: A --> B
    Inheritance,
    /// Similarity: A <-> B
    Similarity,
    /// Implication: A ==> B
    Implication,
    /// Instance: {a} --> B
    Instance,
    /// Property: A --> [b]
    Property,
}

/// Generated Narsese knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarseseKB {
    pub statements: Vec<NarseseStatement>,
}

/// Generates Narsese statements from ontology mappings
pub struct NarseseGenerator {
    default_truth: TruthValue,
}

impl Default for NarseseGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl NarseseGenerator {
    pub fn new() -> Self {
        Self {
            default_truth: TruthValue::extracted(),
        }
    }

    /// Set default truth value for generated statements
    pub fn with_truth_value(mut self, truth: TruthValue) -> Self {
        self.default_truth = truth;
        self
    }

    /// Generate Narsese statements from ontology mapping
    pub fn generate(&self, mapping: &OntologyMapping) -> NarseseKB {
        let mut statements = Vec::new();

        // Generate category inheritance statements
        for element in &mapping.elements {
            statements.extend(self.generate_element_statements(element));
        }

        // Generate design system membership
        if let Some(ds) = mapping.design_system {
            for element in &mapping.elements {
                statements.push(self.generate_ds_membership(element, ds));
            }
        }

        // Generate design token statements
        for (name, value, category) in &mapping.design_tokens {
            statements.push(self.generate_token_statement(name, value, category));
        }

        // Generate category relationships
        statements.extend(self.generate_category_hierarchy(&mapping.categories_used));

        NarseseKB { statements }
    }

    /// Generate statements for a single element
    fn generate_element_statements(&self, element: &MappedElement) -> Vec<NarseseStatement> {
        let mut statements = Vec::new();
        let element_id = self.element_id(element);

        // Instance statement: {element} --> Category
        statements.push(NarseseStatement {
            statement: format!(
                "{{{}}} --> {}. %{:.2};{:.2}%",
                element_id,
                self.category_term(element.category),
                self.default_truth.frequency,
                self.default_truth.confidence
            ),
            truth_value: self.default_truth,
            statement_type: StatementType::Instance,
        });

        // Property statements for classes
        for class in &element.classes {
            statements.push(NarseseStatement {
                statement: format!(
                    "{} --> [{}]. %{:.2};{:.2}%",
                    element_id,
                    self.sanitize_term(class),
                    self.default_truth.frequency,
                    self.default_truth.confidence
                ),
                truth_value: self.default_truth,
                statement_type: StatementType::Property,
            });
        }

        // Interactivity property
        if element.has_interactivity {
            statements.push(NarseseStatement {
                statement: format!(
                    "{} --> [interactive]. %{:.2};{:.2}%",
                    element_id,
                    self.default_truth.frequency,
                    self.default_truth.confidence
                ),
                truth_value: self.default_truth,
                statement_type: StatementType::Property,
            });
        }

        statements
    }

    /// Generate design system membership statement
    fn generate_ds_membership(&self, element: &MappedElement, ds: DesignSystemType) -> NarseseStatement {
        let element_id = self.element_id(element);
        let ds_term = self.ds_term(ds);

        NarseseStatement {
            statement: format!(
                "{{{}}} --> {}. %{:.2};{:.2}%",
                element_id,
                ds_term,
                self.default_truth.frequency,
                self.default_truth.confidence
            ),
            truth_value: self.default_truth,
            statement_type: StatementType::Instance,
        }
    }

    /// Generate design token statement
    fn generate_token_statement(
        &self,
        name: &str,
        value: &str,
        category: &crate::css::TokenCategory,
    ) -> NarseseStatement {
        let token_term = self.sanitize_term(name);
        let category_term = self.token_category_term(*category);

        NarseseStatement {
            statement: format!(
                "<{{{}}} --> {}> && <{{{}}} --> [value_{}]>. %{:.2};{:.2}%",
                token_term,
                category_term,
                token_term,
                self.sanitize_term(value),
                self.default_truth.frequency,
                self.default_truth.confidence
            ),
            truth_value: self.default_truth,
            statement_type: StatementType::Instance,
        }
    }

    /// Generate category hierarchy statements
    fn generate_category_hierarchy(&self, categories: &[UICategory]) -> Vec<NarseseStatement> {
        let mut statements = Vec::new();

        // Define standard hierarchy relationships
        let hierarchy = vec![
            (UICategory::Button, "UIControl"),
            (UICategory::Input, "UIControl"),
            (UICategory::Select, "UIControl"),
            (UICategory::Checkbox, "UIControl"),
            (UICategory::Radio, "UIControl"),
            (UICategory::Switch, "UIControl"),
            (UICategory::Slider, "UIControl"),
            (UICategory::Card, "UIContainer"),
            (UICategory::Modal, "UIOverlay"),
            (UICategory::Dialog, "UIOverlay"),
            (UICategory::Drawer, "UIOverlay"),
            (UICategory::Popover, "UIOverlay"),
            (UICategory::Tooltip, "UIOverlay"),
            (UICategory::Alert, "UIFeedback"),
            (UICategory::Toast, "UIFeedback"),
            (UICategory::Progress, "UIFeedback"),
            (UICategory::Spinner, "UIFeedback"),
            (UICategory::Navigation, "UINavigation"),
            (UICategory::Menu, "UINavigation"),
            (UICategory::Tabs, "UINavigation"),
            (UICategory::Breadcrumb, "UINavigation"),
            (UICategory::Heading, "UITypography"),
            (UICategory::Text, "UITypography"),
            (UICategory::Label, "UITypography"),
        ];

        for (category, parent) in hierarchy {
            if categories.contains(&category) {
                statements.push(NarseseStatement {
                    statement: format!(
                        "{} --> {}. %1.00;0.90%",
                        self.category_term(category),
                        parent
                    ),
                    truth_value: TruthValue::certain(),
                    statement_type: StatementType::Inheritance,
                });
            }
        }

        // Top-level: all UI* --> UIElement
        let top_parents = ["UIControl", "UIContainer", "UIOverlay", "UIFeedback", "UINavigation", "UITypography"];
        for parent in top_parents {
            statements.push(NarseseStatement {
                statement: format!("{} --> UIElement. %1.00;0.90%", parent),
                truth_value: TruthValue::certain(),
                statement_type: StatementType::Inheritance,
            });
        }

        statements
    }

    /// Generate element identifier
    fn element_id(&self, element: &MappedElement) -> String {
        format!(
            "{}_{}",
            self.category_term(element.category),
            element.classes.first().map(|c| self.sanitize_term(c)).unwrap_or_else(|| "default".to_string())
        )
    }

    /// Convert category to Narsese term
    fn category_term(&self, category: UICategory) -> String {
        format!("UI{}", self.capitalize(category.as_str()))
    }

    /// Convert design system to Narsese term
    fn ds_term(&self, ds: DesignSystemType) -> String {
        format!("DS_{}", ds.as_str().replace('-', "_"))
    }

    /// Convert token category to Narsese term
    fn token_category_term(&self, category: crate::css::TokenCategory) -> String {
        match category {
            crate::css::TokenCategory::Color => "DesignToken_Color".to_string(),
            crate::css::TokenCategory::Spacing => "DesignToken_Spacing".to_string(),
            crate::css::TokenCategory::Typography => "DesignToken_Typography".to_string(),
            crate::css::TokenCategory::BorderRadius => "DesignToken_BorderRadius".to_string(),
            crate::css::TokenCategory::Shadow => "DesignToken_Shadow".to_string(),
            crate::css::TokenCategory::Animation => "DesignToken_Animation".to_string(),
            crate::css::TokenCategory::Other => "DesignToken_Other".to_string(),
        }
    }

    /// Sanitize a term for use in Narsese
    fn sanitize_term(&self, term: &str) -> String {
        term.replace('-', "_")
            .replace('.', "_")
            .replace('#', "")
            .replace(' ', "_")
            .replace(':', "_")
            .replace('/', "_")
            .replace('(', "")
            .replace(')', "")
    }

    /// Capitalize first letter
    fn capitalize(&self, s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design_system::DesignSystemDetector;
    use crate::html::HtmlParser;
    use crate::ontology::OntologyMapper;

    #[test]
    fn test_generate_narsese() {
        let mut html_parser = HtmlParser::new();
        let html = html_parser.parse(r#"<button class="btn primary">Click</button>"#).unwrap();

        let detector = DesignSystemDetector::new();
        let ds_result = detector.detect_from_classes(&html.classes);

        let mapper = OntologyMapper::new();
        let mapping = mapper.map_html(&html, &ds_result);

        let generator = NarseseGenerator::new();
        let kb = generator.generate(&mapping);

        assert!(!kb.statements.is_empty());
        assert!(kb.statements.iter().any(|s| s.statement.contains("UIButton")));
    }

    #[test]
    fn test_truth_values() {
        let tv = TruthValue::extracted();
        assert!((tv.frequency - 0.9).abs() < 0.01);
        assert!((tv.confidence - 0.5).abs() < 0.01);
    }
}
