//! Query processor - extracts intent, component type, attributes, context

use regex::Regex;
use serde::{Deserialize, Serialize};

/// User intent extracted from query
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    Create,
    Find,
    Modify,
}

impl Default for Intent {
    fn default() -> Self {
        Self::Find
    }
}

/// Processed query with extracted information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedQuery {
    /// Original query text
    pub original: String,
    /// Detected intent
    pub intent: Intent,
    /// Component types mentioned
    pub component_types: Vec<String>,
    /// Attributes/properties requested
    pub attributes: Vec<String>,
    /// Additional context words
    pub context: Vec<String>,
    /// Search terms for retrieval
    pub search_terms: Vec<String>,
}

/// Extracts structured information from natural language queries
pub struct QueryProcessor {
    intent_patterns: IntentPatterns,
    component_patterns: Vec<(&'static str, Regex)>,
    attribute_patterns: Vec<(&'static str, Regex)>,
}

struct IntentPatterns {
    create: Regex,
    find: Regex,
    modify: Regex,
}

impl Default for QueryProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryProcessor {
    pub fn new() -> Self {
        Self {
            intent_patterns: IntentPatterns {
                create: Regex::new(r"(?i)\b(create|make|build|generate|add|new)\b").unwrap(),
                find: Regex::new(r"(?i)\b(find|search|show|get|list|display|give|need)\b").unwrap(),
                modify: Regex::new(r"(?i)\b(change|modify|update|edit|fix|improve)\b").unwrap(),
            },
            component_patterns: vec![
                ("button", Regex::new(r"(?i)\bbuttons?\b").unwrap()),
                ("form", Regex::new(r"(?i)\bforms?\b").unwrap()),
                ("card", Regex::new(r"(?i)\bcards?\b").unwrap()),
                ("navbar", Regex::new(r"(?i)\b(navbars?|nav|navigation)\b").unwrap()),
                ("modal", Regex::new(r"(?i)\b(modals?|dialogs?|popups?)\b").unwrap()),
                ("table", Regex::new(r"(?i)\btables?\b").unwrap()),
                ("list", Regex::new(r"(?i)\blists?\b").unwrap()),
                ("input", Regex::new(r"(?i)\b(inputs?|text\s*fields?)\b").unwrap()),
                ("dropdown", Regex::new(r"(?i)\b(dropdowns?|selects?)\b").unwrap()),
                ("checkbox", Regex::new(r"(?i)\bcheckbox(es)?\b").unwrap()),
                ("radio", Regex::new(r"(?i)\bradio\s*(buttons?)?\b").unwrap()),
                ("slider", Regex::new(r"(?i)\bsliders?\b").unwrap()),
                ("tabs", Regex::new(r"(?i)\btabs?\b").unwrap()),
                ("accordion", Regex::new(r"(?i)\baccordions?\b").unwrap()),
                ("carousel", Regex::new(r"(?i)\bcarousels?\b").unwrap()),
                ("header", Regex::new(r"(?i)\bheaders?\b").unwrap()),
                ("footer", Regex::new(r"(?i)\bfooters?\b").unwrap()),
                ("sidebar", Regex::new(r"(?i)\bsidebars?\b").unwrap()),
                ("menu", Regex::new(r"(?i)\bmenus?\b").unwrap()),
                ("breadcrumb", Regex::new(r"(?i)\bbreadcrumbs?\b").unwrap()),
                ("pagination", Regex::new(r"(?i)\bpaginations?\b").unwrap()),
                ("tooltip", Regex::new(r"(?i)\btooltips?\b").unwrap()),
                ("toast", Regex::new(r"(?i)\b(toasts?|notifications?)\b").unwrap()),
                ("avatar", Regex::new(r"(?i)\bavatars?\b").unwrap()),
                ("badge", Regex::new(r"(?i)\bbadges?\b").unwrap()),
                ("alert", Regex::new(r"(?i)\balerts?\b").unwrap()),
                ("progress", Regex::new(r"(?i)\b(progress\s*bars?|loaders?)\b").unwrap()),
            ],
            attribute_patterns: vec![
                ("responsive", Regex::new(r"(?i)\bresponsive\b").unwrap()),
                ("dark", Regex::new(r"(?i)\b(dark|dark\s*mode|dark\s*theme)\b").unwrap()),
                ("light", Regex::new(r"(?i)\b(light|light\s*mode|light\s*theme)\b").unwrap()),
                ("animated", Regex::new(r"(?i)\b(animat\w*|transitions?)\b").unwrap()),
                ("accessible", Regex::new(r"(?i)\b(accessib\w*|a11y|aria)\b").unwrap()),
                ("primary", Regex::new(r"(?i)\bprimary\b").unwrap()),
                ("secondary", Regex::new(r"(?i)\bsecondary\b").unwrap()),
                ("large", Regex::new(r"(?i)\b(large|big|xl)\b").unwrap()),
                ("small", Regex::new(r"(?i)\b(small|tiny|xs|mini)\b").unwrap()),
                ("medium", Regex::new(r"(?i)\b(medium|md)\b").unwrap()),
                ("centered", Regex::new(r"(?i)\bcenter(ed)?\b").unwrap()),
                ("rounded", Regex::new(r"(?i)\brounded\b").unwrap()),
                ("outlined", Regex::new(r"(?i)\b(outlined?|bordered?)\b").unwrap()),
                ("filled", Regex::new(r"(?i)\bfilled\b").unwrap()),
                ("disabled", Regex::new(r"(?i)\bdisabled\b").unwrap()),
                ("loading", Regex::new(r"(?i)\bloading\b").unwrap()),
                ("icon", Regex::new(r"(?i)\bicons?\b").unwrap()),
                ("gradient", Regex::new(r"(?i)\bgradients?\b").unwrap()),
                ("shadow", Regex::new(r"(?i)\bshadows?\b").unwrap()),
                ("hover", Regex::new(r"(?i)\bhover\b").unwrap()),
                ("sticky", Regex::new(r"(?i)\bsticky\b").unwrap()),
                ("fixed", Regex::new(r"(?i)\bfixed\b").unwrap()),
                ("fullwidth", Regex::new(r"(?i)\b(full\s*width|full-width)\b").unwrap()),
                ("transparent", Regex::new(r"(?i)\btransparent\b").unwrap()),
            ],
        }
    }

    /// Process a natural language query
    pub fn process(&self, query: &str) -> ProcessedQuery {
        let intent = self.detect_intent(query);
        let component_types = self.extract_components(query);
        let attributes = self.extract_attributes(query);
        let context = self.extract_context(query, &component_types, &attributes);

        // Combine all terms for search
        let mut search_terms = Vec::new();
        search_terms.extend(component_types.clone());
        search_terms.extend(attributes.clone());
        search_terms.extend(context.clone());

        ProcessedQuery {
            original: query.to_string(),
            intent,
            component_types,
            attributes,
            context,
            search_terms,
        }
    }

    fn detect_intent(&self, query: &str) -> Intent {
        if self.intent_patterns.create.is_match(query) {
            Intent::Create
        } else if self.intent_patterns.modify.is_match(query) {
            Intent::Modify
        } else {
            Intent::Find
        }
    }

    fn extract_components(&self, query: &str) -> Vec<String> {
        self.component_patterns
            .iter()
            .filter(|(_, re)| re.is_match(query))
            .map(|(name, _)| name.to_string())
            .collect()
    }

    fn extract_attributes(&self, query: &str) -> Vec<String> {
        self.attribute_patterns
            .iter()
            .filter(|(_, re)| re.is_match(query))
            .map(|(name, _)| name.to_string())
            .collect()
    }

    fn extract_context(&self, query: &str, components: &[String], attributes: &[String]) -> Vec<String> {
        // Extract additional meaningful words not already captured
        let stop_words = [
            "a", "an", "the", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "shall", "can", "need", "dare",
            "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by",
            "from", "as", "into", "through", "during", "before", "after", "above",
            "below", "between", "under", "again", "further", "then", "once", "here",
            "there", "when", "where", "why", "how", "all", "each", "few", "more",
            "most", "other", "some", "such", "no", "nor", "not", "only", "own",
            "same", "so", "than", "too", "very", "just", "and", "but", "if", "or",
            "because", "until", "while", "me", "i", "my", "we", "our", "you", "your",
            "it", "its", "that", "this", "these", "those", "what", "which", "who",
            "create", "make", "build", "generate", "add", "find", "search", "show",
            "get", "list", "display", "give", "change", "modify", "update", "edit",
        ];

        let word_re = Regex::new(r"\b[a-zA-Z]{3,}\b").unwrap();

        word_re
            .find_iter(query)
            .map(|m| m.as_str().to_lowercase())
            .filter(|w| {
                !stop_words.contains(&w.as_str())
                    && !components.contains(w)
                    && !attributes.contains(w)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_query() {
        let processor = QueryProcessor::new();
        let result = processor.process("create a responsive dark mode button");

        assert_eq!(result.intent, Intent::Create);
        assert!(result.component_types.contains(&"button".to_string()));
        assert!(result.attributes.contains(&"responsive".to_string()));
        assert!(result.attributes.contains(&"dark".to_string()));
    }
}
