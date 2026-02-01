//! Narsese language translation and parsing
//!
//! Translates natural language queries to Narsese statements.

use regex::Regex;
use serde::{Deserialize, Serialize};

/// A Narsese statement with truth value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarseseStatement {
    pub statement: String,
    pub frequency: f32,
    pub confidence: f32,
}

impl NarseseStatement {
    pub fn new(statement: impl Into<String>, frequency: f32, confidence: f32) -> Self {
        Self {
            statement: statement.into(),
            frequency,
            confidence,
        }
    }

    /// Format as Narsese input string
    pub fn to_narsese(&self) -> String {
        format!("{} {{{}|{}}}", self.statement, self.frequency, self.confidence)
    }
}

/// Translates natural language to Narsese
pub struct NarseseTranslator {
    // Patterns for intent detection
    create_pattern: Regex,
    find_pattern: Regex,
    modify_pattern: Regex,
}

impl Default for NarseseTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl NarseseTranslator {
    pub fn new() -> Self {
        Self {
            create_pattern: Regex::new(r"(?i)\b(create|make|build|generate|add)\b").unwrap(),
            find_pattern: Regex::new(r"(?i)\b(find|search|show|get|list|display)\b").unwrap(),
            modify_pattern: Regex::new(r"(?i)\b(change|modify|update|edit|fix)\b").unwrap(),
        }
    }

    /// Translate natural language query to Narsese statements
    pub fn translate(&self, query: &str) -> Vec<NarseseStatement> {
        let mut statements = Vec::new();

        // Detect intent
        let intent = self.detect_intent(query);
        statements.push(NarseseStatement::new(
            format!("<query --> [{}]>", intent),
            1.0,
            0.9,
        ));

        // Extract component types mentioned
        let components = self.extract_components(query);
        for component in &components {
            statements.push(NarseseStatement::new(
                format!("<{} --> component>", component),
                1.0,
                0.9,
            ));
            statements.push(NarseseStatement::new(
                format!("<query --> {}>", component),
                1.0,
                0.8,
            ));
        }

        // Extract attributes/properties
        let attributes = self.extract_attributes(query);
        for attr in &attributes {
            statements.push(NarseseStatement::new(
                format!("<query --> [{}]>", attr),
                1.0,
                0.7,
            ));
        }

        // Add relationships between components and attributes
        for component in &components {
            for attr in &attributes {
                statements.push(NarseseStatement::new(
                    format!("<({} * {}) --> has_attribute>", component, attr),
                    1.0,
                    0.6,
                ));
            }
        }

        statements
    }

    /// Detect user intent from query
    pub fn detect_intent(&self, query: &str) -> &'static str {
        if self.create_pattern.is_match(query) {
            "create"
        } else if self.modify_pattern.is_match(query) {
            "modify"
        } else if self.find_pattern.is_match(query) {
            "find"
        } else {
            "find" // Default intent
        }
    }

    /// Extract UI component types from query
    fn extract_components(&self, query: &str) -> Vec<String> {
        let component_patterns = [
            ("button", r"(?i)\bbutton\b"),
            ("form", r"(?i)\bform\b"),
            ("card", r"(?i)\bcard\b"),
            ("navbar", r"(?i)\b(navbar|nav|navigation)\b"),
            ("modal", r"(?i)\b(modal|dialog|popup)\b"),
            ("table", r"(?i)\btable\b"),
            ("list", r"(?i)\blist\b"),
            ("input", r"(?i)\b(input|text\s*field)\b"),
            ("dropdown", r"(?i)\b(dropdown|select)\b"),
            ("checkbox", r"(?i)\bcheckbox\b"),
            ("radio", r"(?i)\bradio\b"),
            ("slider", r"(?i)\bslider\b"),
            ("tabs", r"(?i)\btabs?\b"),
            ("accordion", r"(?i)\baccordion\b"),
            ("carousel", r"(?i)\bcarousel\b"),
            ("header", r"(?i)\bheader\b"),
            ("footer", r"(?i)\bfooter\b"),
            ("sidebar", r"(?i)\bsidebar\b"),
        ];

        let mut found = Vec::new();
        for (name, pattern) in component_patterns {
            if Regex::new(pattern).unwrap().is_match(query) {
                found.push(name.to_string());
            }
        }

        if found.is_empty() {
            // Try to extract generic component mention
            found.push("component".to_string());
        }

        found
    }

    /// Extract attributes/properties from query
    fn extract_attributes(&self, query: &str) -> Vec<String> {
        let attribute_patterns = [
            ("responsive", r"(?i)\bresponsive\b"),
            ("dark", r"(?i)\b(dark|dark\s*mode)\b"),
            ("light", r"(?i)\b(light|light\s*mode)\b"),
            ("animated", r"(?i)\b(animat|transition)\b"),
            ("accessible", r"(?i)\b(accessib|a11y|aria)\b"),
            ("primary", r"(?i)\bprimary\b"),
            ("secondary", r"(?i)\bsecondary\b"),
            ("large", r"(?i)\b(large|big)\b"),
            ("small", r"(?i)\b(small|tiny)\b"),
            ("centered", r"(?i)\bcenter\b"),
            ("rounded", r"(?i)\brounded\b"),
            ("outlined", r"(?i)\b(outlined?|border)\b"),
            ("filled", r"(?i)\bfilled\b"),
            ("disabled", r"(?i)\bdisabled\b"),
            ("loading", r"(?i)\bloading\b"),
            ("icon", r"(?i)\bicon\b"),
        ];

        let mut found = Vec::new();
        for (name, pattern) in attribute_patterns {
            if Regex::new(pattern).unwrap().is_match(query) {
                found.push(name.to_string());
            }
        }

        found
    }
}

/// Parse ONA response to extract derived statements
pub fn parse_ona_response(response: &str) -> Vec<NarseseStatement> {
    let mut statements = Vec::new();

    // Pattern for ONA output: Answer: <statement>. creationTime=X Truth: frequency=F confidence=C
    let answer_re =
        Regex::new(r"Answer:\s*(.+?)\.\s*creationTime=\d+\s*Truth:\s*frequency=([\d.]+)\s*confidence=([\d.]+)")
            .unwrap();

    for cap in answer_re.captures_iter(response) {
        let statement = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let frequency: f32 = cap
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0.0);
        let confidence: f32 = cap
            .get(3)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0.0);

        if !statement.is_empty() {
            statements.push(NarseseStatement::new(statement, frequency, confidence));
        }
    }

    // Also capture derived statements
    let derived_re = Regex::new(r"Derived:\s*(.+?)\s*Truth:\s*frequency=([\d.]+)\s*confidence=([\d.]+)")
        .unwrap();

    for cap in derived_re.captures_iter(response) {
        let statement = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let frequency: f32 = cap
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0.0);
        let confidence: f32 = cap
            .get(3)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0.0);

        if !statement.is_empty() {
            statements.push(NarseseStatement::new(statement, frequency, confidence));
        }
    }

    statements
}

/// Extract search terms from Narsese statements
pub fn extract_search_terms(statements: &[NarseseStatement]) -> Vec<String> {
    let mut terms = Vec::new();
    let term_re = Regex::new(r"<(\w+)\s*-->").unwrap();
    let attr_re = Regex::new(r"\[(\w+)\]").unwrap();

    for stmt in statements {
        // Extract subjects
        for cap in term_re.captures_iter(&stmt.statement) {
            if let Some(term) = cap.get(1) {
                let t = term.as_str();
                if t != "query" && !terms.contains(&t.to_string()) {
                    terms.push(t.to_string());
                }
            }
        }

        // Extract attributes in brackets
        for cap in attr_re.captures_iter(&stmt.statement) {
            if let Some(attr) = cap.get(1) {
                let a = attr.as_str();
                if !terms.contains(&a.to_string()) {
                    terms.push(a.to_string());
                }
            }
        }
    }

    terms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_query() {
        let translator = NarseseTranslator::new();
        let statements = translator.translate("create a responsive button with dark mode");

        assert!(!statements.is_empty());

        // Should detect create intent
        assert!(statements
            .iter()
            .any(|s| s.statement.contains("[create]")));

        // Should extract button component
        assert!(statements.iter().any(|s| s.statement.contains("button")));
    }

    #[test]
    fn test_extract_search_terms() {
        let statements = vec![
            NarseseStatement::new("<button --> component>", 1.0, 0.9),
            NarseseStatement::new("<query --> [responsive]>", 1.0, 0.7),
        ];

        let terms = extract_search_terms(&statements);
        assert!(terms.contains(&"button".to_string()));
        assert!(terms.contains(&"responsive".to_string()));
    }
}
