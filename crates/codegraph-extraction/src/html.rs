//! HTML AST parser - extracts tags, attributes, classes, and structure

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser};

static HTML_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_html::LANGUAGE.into());

/// Extracted HTML element information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlElement {
    /// Tag name (e.g., "div", "button", "form")
    pub tag: String,
    /// Element ID if present
    pub id: Option<String>,
    /// CSS classes
    pub classes: Vec<String>,
    /// All attributes as key-value pairs
    pub attributes: Vec<(String, String)>,
    /// Child elements
    pub children: Vec<HtmlElement>,
    /// Text content (for leaf elements)
    pub text_content: Option<String>,
    /// Depth in the DOM tree
    pub depth: usize,
}

/// Extracted structure from HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlStructure {
    /// Root elements
    pub elements: Vec<HtmlElement>,
    /// All unique tags found
    pub tags: Vec<String>,
    /// All unique classes found
    pub classes: Vec<String>,
    /// All unique IDs found
    pub ids: Vec<String>,
}

/// HTML parser using tree-sitter
pub struct HtmlParser {
    parser: Parser,
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&HTML_LANGUAGE)
            .expect("Failed to set HTML language");
        Self { parser }
    }

    /// Parse HTML string and extract structure
    pub fn parse(&mut self, html: &str) -> anyhow::Result<HtmlStructure> {
        let tree = self
            .parser
            .parse(html, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse HTML"))?;

        let root = tree.root_node();
        let mut elements = Vec::new();
        let mut all_tags = Vec::new();
        let mut all_classes = Vec::new();
        let mut all_ids = Vec::new();

        self.extract_elements(root, html, 0, &mut elements, &mut all_tags, &mut all_classes, &mut all_ids);

        // Deduplicate
        all_tags.sort();
        all_tags.dedup();
        all_classes.sort();
        all_classes.dedup();
        all_ids.sort();
        all_ids.dedup();

        Ok(HtmlStructure {
            elements,
            tags: all_tags,
            classes: all_classes,
            ids: all_ids,
        })
    }

    fn extract_elements(
        &self,
        node: Node,
        source: &str,
        depth: usize,
        elements: &mut Vec<HtmlElement>,
        all_tags: &mut Vec<String>,
        all_classes: &mut Vec<String>,
        all_ids: &mut Vec<String>,
    ) {
        if node.kind() == "element" {
            if let Some(element) = self.extract_element(node, source, depth, all_tags, all_classes, all_ids) {
                elements.push(element);
            }
        } else {
            // Recurse into children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.extract_elements(child, source, depth, elements, all_tags, all_classes, all_ids);
            }
        }
    }

    fn extract_element(
        &self,
        node: Node,
        source: &str,
        depth: usize,
        all_tags: &mut Vec<String>,
        all_classes: &mut Vec<String>,
        all_ids: &mut Vec<String>,
    ) -> Option<HtmlElement> {
        let mut tag = String::new();
        let mut id = None;
        let mut classes = Vec::new();
        let mut attributes = Vec::new();
        let mut children = Vec::new();
        let mut text_content = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "start_tag" | "self_closing_tag" => {
                    // Extract tag name and attributes
                    let mut tag_cursor = child.walk();
                    for tag_child in child.children(&mut tag_cursor) {
                        match tag_child.kind() {
                            "tag_name" => {
                                tag = self.node_text(tag_child, source).to_lowercase();
                                all_tags.push(tag.clone());
                            }
                            "attribute" => {
                                if let Some((name, value)) = self.extract_attribute(tag_child, source) {
                                    if name == "id" {
                                        id = Some(value.clone());
                                        all_ids.push(value.clone());
                                    } else if name == "class" {
                                        let class_list: Vec<String> = value
                                            .split_whitespace()
                                            .map(|s| s.to_string())
                                            .collect();
                                        all_classes.extend(class_list.clone());
                                        classes = class_list;
                                    }
                                    attributes.push((name, value));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "element" => {
                    // Recurse for child elements
                    if let Some(child_elem) = self.extract_element(child, source, depth + 1, all_tags, all_classes, all_ids) {
                        children.push(child_elem);
                    }
                }
                "text" => {
                    let text = self.node_text(child, source).trim().to_string();
                    if !text.is_empty() {
                        text_content = Some(text);
                    }
                }
                _ => {}
            }
        }

        if tag.is_empty() {
            return None;
        }

        Some(HtmlElement {
            tag,
            id,
            classes,
            attributes,
            children,
            text_content,
            depth,
        })
    }

    fn extract_attribute(&self, node: Node, source: &str) -> Option<(String, String)> {
        let mut name = String::new();
        let mut value = String::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "attribute_name" => {
                    name = self.node_text(child, source).to_lowercase();
                }
                "quoted_attribute_value" | "attribute_value" => {
                    value = self.node_text(child, source)
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                }
                _ => {}
            }
        }

        if name.is_empty() {
            None
        } else {
            Some((name, value))
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
    fn test_parse_html() {
        let mut parser = HtmlParser::new();
        let html = r#"<div class="container"><button id="submit" class="btn primary">Click</button></div>"#;

        let result = parser.parse(html).unwrap();

        assert!(result.tags.contains(&"div".to_string()));
        assert!(result.tags.contains(&"button".to_string()));
        assert!(result.classes.contains(&"container".to_string()));
        assert!(result.classes.contains(&"btn".to_string()));
        assert!(result.ids.contains(&"submit".to_string()));
    }
}
