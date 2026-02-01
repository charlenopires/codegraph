//! System prompts for LLM code generation

/// System prompt for vanilla HTML/CSS/JS code generation
pub const SYSTEM_PROMPT: &str = r#"You are an expert frontend developer specializing in vanilla HTML5, CSS3, and ES6+ JavaScript.

## Code Standards

### HTML5
- Use semantic elements: <header>, <nav>, <main>, <section>, <article>, <aside>, <footer>
- Include proper ARIA attributes for accessibility (role, aria-label, aria-describedby)
- Use <button> for interactive elements, not <div> with onclick
- Include lang attribute on <html>
- Use proper heading hierarchy (h1-h6)

### CSS3
- Use CSS custom properties (--variable-name) for theming
- Implement responsive design with CSS Grid and Flexbox
- Use relative units (rem, em, %) over fixed pixels
- Include focus states for keyboard navigation
- Use media queries for breakpoints:
  - Mobile: max-width: 480px
  - Tablet: max-width: 768px
  - Desktop: min-width: 1024px

### JavaScript ES6+
- Use const/let, never var
- Use arrow functions for callbacks
- Use template literals for string interpolation
- Use destructuring where appropriate
- Use async/await for asynchronous operations
- Add event listeners with addEventListener, not inline handlers

## Output Format

Return code in three separate blocks:

```html
<!-- Your HTML here -->
```

```css
/* Your CSS here */
```

```javascript
// Your JavaScript here
```

## Comments
Include inline comments explaining:
- Structure decisions
- Accessibility considerations
- Responsive breakpoints
- JavaScript functionality
"#;

/// Build user prompt with graph context
pub fn build_user_prompt(
    description: &str,
    similar_elements: &[SimilarElement],
    templates: &[TemplateContext],
    reasoning: Option<&str>,
) -> String {
    let mut prompt = format!("Generate a UI component: {}\n\n", description);

    if !similar_elements.is_empty() {
        prompt.push_str("## Similar Elements from Knowledge Graph\n\n");
        for (i, elem) in similar_elements.iter().take(5).enumerate() {
            prompt.push_str(&format!(
                "{}. **{}** (similarity: {:.2})\n   - Category: {}\n   - Tags: {}\n\n",
                i + 1,
                elem.name,
                elem.similarity,
                elem.category,
                elem.tags.join(", ")
            ));
        }
    }

    if !templates.is_empty() {
        prompt.push_str("## Available Templates\n\n");
        for template in templates {
            prompt.push_str(&format!(
                "### {}\n```html\n{}\n```\n\n",
                template.name, template.html
            ));
        }
    }

    if let Some(reasoning) = reasoning {
        prompt.push_str(&format!("## NARS Reasoning Output\n\n{}\n\n", reasoning));
    }

    prompt.push_str("Generate the component following all standards above.");
    prompt
}

/// Similar element from graph retrieval
#[derive(Debug, Clone)]
pub struct SimilarElement {
    pub name: String,
    pub category: String,
    pub tags: Vec<String>,
    pub similarity: f32,
}

/// Template context for generation
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub name: String,
    pub html: String,
}
