//! JavaScript AST parser - extracts functions, event handlers, and dependencies

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser};

static JS_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_javascript::LANGUAGE.into());

/// JavaScript function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsFunction {
    pub name: String,
    pub params: Vec<String>,
    pub is_async: bool,
    pub is_arrow: bool,
    pub body_preview: Option<String>,
}

/// Event handler binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandler {
    pub event_type: String,
    pub handler_name: Option<String>,
    pub target_selector: Option<String>,
}

/// Import/dependency reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsImport {
    pub source: String,
    pub specifiers: Vec<String>,
    pub is_default: bool,
}

/// Extracted JavaScript structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsStructure {
    /// All function definitions
    pub functions: Vec<JsFunction>,
    /// Event handlers found
    pub event_handlers: Vec<EventHandler>,
    /// Import statements
    pub imports: Vec<JsImport>,
    /// Variable declarations
    pub variables: Vec<String>,
    /// DOM API calls detected
    pub dom_calls: Vec<String>,
}

/// JavaScript parser using tree-sitter
pub struct JsParser {
    parser: Parser,
}

impl Default for JsParser {
    fn default() -> Self {
        Self::new()
    }
}

impl JsParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&JS_LANGUAGE)
            .expect("Failed to set JavaScript language");
        Self { parser }
    }

    /// Parse JavaScript string and extract structure
    pub fn parse(&mut self, js: &str) -> anyhow::Result<JsStructure> {
        let tree = self
            .parser
            .parse(js, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse JavaScript"))?;

        let root = tree.root_node();
        let mut functions = Vec::new();
        let mut event_handlers = Vec::new();
        let mut imports = Vec::new();
        let mut variables = Vec::new();
        let mut dom_calls = Vec::new();

        self.extract_nodes(
            root,
            js,
            &mut functions,
            &mut event_handlers,
            &mut imports,
            &mut variables,
            &mut dom_calls,
        );

        Ok(JsStructure {
            functions,
            event_handlers,
            imports,
            variables,
            dom_calls,
        })
    }

    fn extract_nodes(
        &self,
        node: Node,
        source: &str,
        functions: &mut Vec<JsFunction>,
        event_handlers: &mut Vec<EventHandler>,
        imports: &mut Vec<JsImport>,
        variables: &mut Vec<String>,
        dom_calls: &mut Vec<String>,
    ) {
        match node.kind() {
            "function_declaration" => {
                if let Some(func) = self.extract_function(node, source, false) {
                    functions.push(func);
                }
            }
            "arrow_function" => {
                if let Some(func) = self.extract_arrow_function(node, source) {
                    functions.push(func);
                }
            }
            "variable_declaration" => {
                self.extract_variable(node, source, variables, functions);
            }
            "import_statement" => {
                if let Some(import) = self.extract_import(node, source) {
                    imports.push(import);
                }
            }
            "call_expression" => {
                self.extract_call(node, source, event_handlers, dom_calls);
            }
            "assignment_expression" => {
                self.extract_event_assignment(node, source, event_handlers);
            }
            _ => {}
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_nodes(
                child,
                source,
                functions,
                event_handlers,
                imports,
                variables,
                dom_calls,
            );
        }
    }

    fn extract_function(&self, node: Node, source: &str, is_async: bool) -> Option<JsFunction> {
        let mut name = String::new();
        let mut params = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    name = self.node_text(child, source).to_string();
                }
                "formal_parameters" => {
                    params = self.extract_params(child, source);
                }
                "async" => {
                    return self.extract_function(node, source, true);
                }
                _ => {}
            }
        }

        if name.is_empty() {
            return None;
        }

        Some(JsFunction {
            name,
            params,
            is_async,
            is_arrow: false,
            body_preview: None,
        })
    }

    fn extract_arrow_function(&self, node: Node, source: &str) -> Option<JsFunction> {
        let mut params = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "formal_parameters" => {
                    params = self.extract_params(child, source);
                }
                "identifier" => {
                    // Single parameter arrow function: x => x + 1
                    params.push(self.node_text(child, source).to_string());
                }
                _ => {}
            }
        }

        Some(JsFunction {
            name: "<arrow>".to_string(),
            params,
            is_async: false,
            is_arrow: true,
            body_preview: None,
        })
    }

    fn extract_params(&self, node: Node, source: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "pattern" {
                params.push(self.node_text(child, source).to_string());
            }
        }

        params
    }

    fn extract_variable(
        &self,
        node: Node,
        source: &str,
        variables: &mut Vec<String>,
        functions: &mut Vec<JsFunction>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                let mut var_cursor = child.walk();
                let mut var_name = String::new();
                let mut is_func = false;

                for var_child in child.children(&mut var_cursor) {
                    match var_child.kind() {
                        "identifier" => {
                            var_name = self.node_text(var_child, source).to_string();
                        }
                        "arrow_function" | "function_expression" => {
                            is_func = true;
                            if !var_name.is_empty() {
                                functions.push(JsFunction {
                                    name: var_name.clone(),
                                    params: Vec::new(),
                                    is_async: false,
                                    is_arrow: var_child.kind() == "arrow_function",
                                    body_preview: None,
                                });
                            }
                        }
                        _ => {}
                    }
                }

                if !is_func && !var_name.is_empty() {
                    variables.push(var_name);
                }
            }
        }
    }

    fn extract_import(&self, node: Node, source: &str) -> Option<JsImport> {
        let mut source_module = String::new();
        let mut specifiers = Vec::new();
        let mut is_default = false;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "string" => {
                    source_module = self
                        .node_text(child, source)
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                }
                "import_clause" => {
                    let mut clause_cursor = child.walk();
                    for clause_child in child.children(&mut clause_cursor) {
                        match clause_child.kind() {
                            "identifier" => {
                                is_default = true;
                                specifiers.push(self.node_text(clause_child, source).to_string());
                            }
                            "named_imports" => {
                                let mut imports_cursor = clause_child.walk();
                                for import_child in clause_child.children(&mut imports_cursor) {
                                    if import_child.kind() == "import_specifier" {
                                        let mut spec_cursor = import_child.walk();
                                        for spec_child in import_child.children(&mut spec_cursor) {
                                            if spec_child.kind() == "identifier" {
                                                specifiers.push(
                                                    self.node_text(spec_child, source).to_string(),
                                                );
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        if source_module.is_empty() {
            return None;
        }

        Some(JsImport {
            source: source_module,
            specifiers,
            is_default,
        })
    }

    fn extract_call(
        &self,
        node: Node,
        source: &str,
        event_handlers: &mut Vec<EventHandler>,
        dom_calls: &mut Vec<String>,
    ) {
        let call_text = self.node_text(node, source);

        // Detect addEventListener calls
        if call_text.contains("addEventListener") {
            if let Some(handler) = self.parse_add_event_listener(node, source) {
                event_handlers.push(handler);
            }
        }

        // Detect DOM API calls
        let dom_patterns = [
            "querySelector",
            "querySelectorAll",
            "getElementById",
            "getElementsByClassName",
            "getElementsByTagName",
            "createElement",
            "appendChild",
            "removeChild",
            "insertBefore",
            "setAttribute",
            "getAttribute",
            "classList",
            "innerHTML",
            "textContent",
        ];

        for pattern in dom_patterns {
            if call_text.contains(pattern) {
                dom_calls.push(pattern.to_string());
            }
        }
    }

    fn parse_add_event_listener(&self, node: Node, source: &str) -> Option<EventHandler> {
        let mut event_type = String::new();
        let mut handler_name = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "arguments" {
                let mut arg_cursor = child.walk();
                let mut arg_index = 0;

                for arg_child in child.children(&mut arg_cursor) {
                    match arg_child.kind() {
                        "string" => {
                            if arg_index == 0 {
                                event_type = self
                                    .node_text(arg_child, source)
                                    .trim_matches('"')
                                    .trim_matches('\'')
                                    .to_string();
                            }
                            arg_index += 1;
                        }
                        "identifier" => {
                            if arg_index == 1 {
                                handler_name = Some(self.node_text(arg_child, source).to_string());
                            }
                            arg_index += 1;
                        }
                        "arrow_function" | "function_expression" => {
                            handler_name = Some("<anonymous>".to_string());
                            arg_index += 1;
                        }
                        _ => {}
                    }
                }
            }
        }

        if event_type.is_empty() {
            return None;
        }

        Some(EventHandler {
            event_type,
            handler_name,
            target_selector: None,
        })
    }

    fn extract_event_assignment(&self, node: Node, source: &str, event_handlers: &mut Vec<EventHandler>) {
        let text = self.node_text(node, source);

        // Detect onclick, onchange, etc. assignments
        let event_props = [
            "onclick",
            "onchange",
            "onsubmit",
            "onkeydown",
            "onkeyup",
            "onmouseover",
            "onmouseout",
            "onfocus",
            "onblur",
            "oninput",
        ];

        for prop in event_props {
            if text.contains(prop) {
                let event_type = prop.strip_prefix("on").unwrap_or(prop).to_string();
                event_handlers.push(EventHandler {
                    event_type,
                    handler_name: None,
                    target_selector: None,
                });
                break;
            }
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
    fn test_parse_javascript() {
        let mut parser = JsParser::new();
        let js = r#"
            import { useState } from 'react';

            function handleClick(event) {
                console.log('clicked');
            }

            const btn = document.querySelector('.btn');
            btn.addEventListener('click', handleClick);
        "#;

        let result = parser.parse(js).unwrap();

        assert!(result.functions.iter().any(|f| f.name == "handleClick"));
        assert!(result.imports.iter().any(|i| i.source == "react"));
        assert!(result.event_handlers.iter().any(|e| e.event_type == "click"));
        assert!(result.dom_calls.contains(&"querySelector".to_string()));
    }
}
