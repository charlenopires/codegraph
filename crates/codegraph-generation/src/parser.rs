//! Code parser - extracts HTML, CSS, JS blocks from LLM response

use regex::Regex;

/// Parsed code blocks from LLM response
#[derive(Debug, Clone, Default)]
pub struct ParsedCode {
    pub html: Option<String>,
    pub css: Option<String>,
    pub javascript: Option<String>,
}

impl ParsedCode {
    /// Check if any code was parsed
    pub fn is_empty(&self) -> bool {
        self.html.is_none() && self.css.is_none() && self.javascript.is_none()
    }

    /// Combine into a single HTML document
    pub fn to_html_document(&self) -> String {
        let mut doc = String::from("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        doc.push_str("  <meta charset=\"UTF-8\">\n");
        doc.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        doc.push_str("  <title>Generated Component</title>\n");

        if let Some(css) = &self.css {
            doc.push_str("  <style>\n");
            for line in css.lines() {
                doc.push_str("    ");
                doc.push_str(line);
                doc.push('\n');
            }
            doc.push_str("  </style>\n");
        }

        doc.push_str("</head>\n<body>\n");

        if let Some(html) = &self.html {
            for line in html.lines() {
                doc.push_str("  ");
                doc.push_str(line);
                doc.push('\n');
            }
        }

        if let Some(js) = &self.javascript {
            doc.push_str("  <script>\n");
            for line in js.lines() {
                doc.push_str("    ");
                doc.push_str(line);
                doc.push('\n');
            }
            doc.push_str("  </script>\n");
        }

        doc.push_str("</body>\n</html>");
        doc
    }
}

/// Parser for extracting code blocks from LLM response
pub struct CodeParser {
    html_regex: Regex,
    css_regex: Regex,
    js_regex: Regex,
}

impl Default for CodeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeParser {
    pub fn new() -> Self {
        Self {
            html_regex: Regex::new(r"```html\s*\n([\s\S]*?)```").unwrap(),
            css_regex: Regex::new(r"```css\s*\n([\s\S]*?)```").unwrap(),
            js_regex: Regex::new(r"```(?:javascript|js)\s*\n([\s\S]*?)```").unwrap(),
        }
    }

    /// Parse LLM response and extract code blocks
    pub fn parse(&self, response: &str) -> ParsedCode {
        ParsedCode {
            html: self.extract_block(&self.html_regex, response),
            css: self.extract_block(&self.css_regex, response),
            javascript: self.extract_block(&self.js_regex, response),
        }
    }

    fn extract_block(&self, regex: &Regex, text: &str) -> Option<String> {
        regex
            .captures(text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Validate HTML is well-formed (basic check)
    pub fn validate_html(&self, html: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check for unclosed tags (simple heuristic)
        let open_tags: Vec<&str> = vec![
            "<div", "<span", "<p", "<section", "<article", "<header", "<footer", "<nav", "<main",
            "<aside", "<form", "<button", "<ul", "<ol", "<li", "<table", "<thead", "<tbody", "<tr",
            "<td", "<th",
        ];

        for tag in open_tags {
            let tag_name = &tag[1..];
            let open_count = html.matches(tag).count();
            let close_pattern = format!("</{}", tag_name);
            let close_count = html.matches(&close_pattern).count();

            // Self-closing or void elements don't need closing tags
            if tag_name != "br" && tag_name != "hr" && tag_name != "img" && tag_name != "input" {
                if open_count > close_count {
                    errors.push(format!(
                        "Potentially unclosed <{}> tag: {} open, {} close",
                        tag_name, open_count, close_count
                    ));
                }
            }
        }

        // Check for DOCTYPE in full documents
        if html.contains("<html") && !html.to_lowercase().contains("<!doctype") {
            errors.push("Missing <!DOCTYPE html> declaration".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_code_blocks() {
        let response = r#"
Here's your button component:

```html
<button class="btn">Click me</button>
```

```css
.btn { color: blue; }
```

```javascript
console.log('hello');
```
"#;

        let parser = CodeParser::new();
        let parsed = parser.parse(response);

        assert_eq!(
            parsed.html,
            Some("<button class=\"btn\">Click me</button>".to_string())
        );
        assert_eq!(parsed.css, Some(".btn { color: blue; }".to_string()));
        assert_eq!(parsed.javascript, Some("console.log('hello');".to_string()));
    }
}
