//! CSS AST parser - extracts rules, properties, and design tokens

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser};

static CSS_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_css::LANGUAGE.into());

/// CSS custom property (design token)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignToken {
    pub name: String,
    pub value: String,
    pub category: TokenCategory,
}

/// Token category for design tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenCategory {
    Color,
    Spacing,
    Typography,
    BorderRadius,
    Shadow,
    Animation,
    Other,
}

/// CSS rule with selector and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssRule {
    pub selector: String,
    pub properties: Vec<CssProperty>,
}

/// CSS property declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssProperty {
    pub name: String,
    pub value: String,
}

/// Extracted CSS structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssStructure {
    /// All CSS rules
    pub rules: Vec<CssRule>,
    /// Detected design tokens (CSS custom properties)
    pub design_tokens: Vec<DesignToken>,
    /// All unique property names
    pub properties: Vec<String>,
    /// All selectors used
    pub selectors: Vec<String>,
}

/// CSS parser using tree-sitter
pub struct CssParser {
    parser: Parser,
    color_re: Regex,
    spacing_re: Regex,
}

impl Default for CssParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CssParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&CSS_LANGUAGE)
            .expect("Failed to set CSS language");

        Self {
            parser,
            color_re: Regex::new(r"(?i)(color|background|border-color|fill|stroke)").unwrap(),
            spacing_re: Regex::new(r"(?i)(margin|padding|gap|space|size|width|height)").unwrap(),
        }
    }

    /// Parse CSS string and extract structure
    pub fn parse(&mut self, css: &str) -> anyhow::Result<CssStructure> {
        let tree = self
            .parser
            .parse(css, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse CSS"))?;

        let root = tree.root_node();
        let mut rules = Vec::new();
        let mut design_tokens = Vec::new();
        let mut all_properties = Vec::new();
        let mut all_selectors = Vec::new();

        self.extract_rules(root, css, &mut rules, &mut design_tokens, &mut all_properties, &mut all_selectors);

        all_properties.sort();
        all_properties.dedup();
        all_selectors.sort();
        all_selectors.dedup();

        Ok(CssStructure {
            rules,
            design_tokens,
            properties: all_properties,
            selectors: all_selectors,
        })
    }

    fn extract_rules(
        &self,
        node: Node,
        source: &str,
        rules: &mut Vec<CssRule>,
        tokens: &mut Vec<DesignToken>,
        all_properties: &mut Vec<String>,
        all_selectors: &mut Vec<String>,
    ) {
        match node.kind() {
            "rule_set" => {
                if let Some(rule) = self.extract_rule(node, source, tokens, all_properties) {
                    all_selectors.push(rule.selector.clone());
                    rules.push(rule);
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_rules(child, source, rules, tokens, all_properties, all_selectors);
                }
            }
        }
    }

    fn extract_rule(
        &self,
        node: Node,
        source: &str,
        tokens: &mut Vec<DesignToken>,
        all_properties: &mut Vec<String>,
    ) -> Option<CssRule> {
        let mut selector = String::new();
        let mut properties = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "selectors" => {
                    selector = self.node_text(child, source).trim().to_string();
                }
                "block" => {
                    self.extract_declarations(child, source, &mut properties, tokens, all_properties);
                }
                _ => {}
            }
        }

        if selector.is_empty() {
            return None;
        }

        Some(CssRule { selector, properties })
    }

    fn extract_declarations(
        &self,
        node: Node,
        source: &str,
        properties: &mut Vec<CssProperty>,
        tokens: &mut Vec<DesignToken>,
        all_properties: &mut Vec<String>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "declaration" {
                if let Some((name, value)) = self.extract_declaration(child, source) {
                    all_properties.push(name.clone());

                    // Check for CSS custom property (design token)
                    if name.starts_with("--") {
                        let category = self.categorize_token(&name, &value);
                        tokens.push(DesignToken {
                            name: name.clone(),
                            value: value.clone(),
                            category,
                        });
                    }

                    properties.push(CssProperty { name, value });
                }
            }
        }
    }

    fn extract_declaration(&self, node: Node, source: &str) -> Option<(String, String)> {
        let mut name = String::new();
        let mut value = String::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "property_name" => {
                    name = self.node_text(child, source).trim().to_string();
                }
                _ if child.kind().contains("value") || child.kind() == "plain_value" => {
                    value = self.node_text(child, source).trim().to_string();
                }
                _ => {}
            }
        }

        // Fallback: extract value from the full declaration
        if value.is_empty() && !name.is_empty() {
            let full = self.node_text(node, source);
            if let Some(colon_pos) = full.find(':') {
                value = full[colon_pos + 1..].trim().trim_end_matches(';').to_string();
            }
        }

        if name.is_empty() {
            None
        } else {
            Some((name, value))
        }
    }

    fn categorize_token(&self, name: &str, value: &str) -> TokenCategory {
        let name_lower = name.to_lowercase();
        let value_lower = value.to_lowercase();

        if self.color_re.is_match(&name_lower) || value_lower.starts_with('#') || value_lower.starts_with("rgb") || value_lower.starts_with("hsl") {
            TokenCategory::Color
        } else if self.spacing_re.is_match(&name_lower) || value_lower.ends_with("px") || value_lower.ends_with("rem") || value_lower.ends_with("em") {
            TokenCategory::Spacing
        } else if name_lower.contains("font") || name_lower.contains("text") || name_lower.contains("line-height") {
            TokenCategory::Typography
        } else if name_lower.contains("radius") {
            TokenCategory::BorderRadius
        } else if name_lower.contains("shadow") {
            TokenCategory::Shadow
        } else if name_lower.contains("animation") || name_lower.contains("transition") {
            TokenCategory::Animation
        } else {
            TokenCategory::Other
        }
    }

    fn node_text<'a>(&self, node: Node, source: &'a str) -> &'a str {
        &source[node.start_byte()..node.end_byte()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_css() {
        let mut parser = CssParser::new();
        let css = r#"
            :root {
                --color-primary: #3b82f6;
                --spacing-md: 1rem;
            }
            .btn {
                background: var(--color-primary);
                padding: var(--spacing-md);
            }
        "#;

        let result = parser.parse(css).unwrap();

        assert!(result.design_tokens.iter().any(|t| t.name == "--color-primary"));
        assert!(result.selectors.contains(&".btn".to_string()));
    }
}
