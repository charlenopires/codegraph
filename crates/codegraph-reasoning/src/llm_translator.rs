//! LLM-based Narsese translator
//!
//! Uses GPT-4o-mini for NL↔Narsese translation with specialized prompts.

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use tracing::{debug, info};

use crate::narsese::NarseseStatement;

/// System prompt for NL→Narsese translation
const NL_TO_NARSESE_PROMPT: &str = r#"You are an expert in NARS (Non-Axiomatic Reasoning System) and Narsese syntax.

## Narsese Syntax Reference

### Statement Types
- Inheritance: <subject --> predicate>
- Similarity: <subject <-> predicate>
- Implication: <antecedent ==> consequent>
- Equivalence: <statement1 <=> statement2>
- Instance: <instance {-- class>
- Property: <object --] property>
- Instance-Property: <object {-] property>

### Compound Terms
- Extensional set: {a, b, c}
- Intensional set: [a, b, c]
- Extensional intersection: (& a b)
- Intensional intersection: (| a b)
- Extensional difference: (- a b)
- Intensional difference: (~ a b)
- Product: (* a b c)
- Extensional image: (/ R _ a) or (/ R a _)
- Intensional image: (\ R _ a) or (\ R a _)
- Negation: (-- statement)
- Conjunction: (&& a b c)
- Disjunction: (|| a b c)

### Truth Values
- Format: {frequency|confidence}
- Example: <cat --> animal>. {1.0|0.9}

## UI/UX Domain Knowledge

Components: button, form, card, navbar, modal, input, dropdown, table, list, tabs
Attributes: responsive, animated, accessible, dark, light, primary, secondary, large, small
Relationships: contains, triggers, displays, validates, submits

## Output Format

Return ONLY Narsese statements, one per line, with truth values.
No explanations or markdown formatting.

Example output:
<button --> component>. {1.0|0.9}
<query --> [create]>. {1.0|0.9}
<(button * primary) --> requested>. {1.0|0.8}
"#;

/// System prompt for Narsese→NL translation
const NARSESE_TO_NL_PROMPT: &str = r#"You are an expert in NARS (Non-Axiomatic Reasoning System) and Narsese syntax.

Convert Narsese statements to natural language explanations for UI developers.

## Guidelines
- Explain what each statement means in plain English
- Focus on the UI/UX implications
- Include confidence levels when significant
- Keep explanations concise but informative

## Example
Input: <button --> [responsive]>. {0.9|0.8}
Output: The button should be responsive (90% certainty, 80% confidence based on evidence).

Input: <(form * validation) --> required>. {1.0|0.95}
Output: The form requires validation with high confidence.
"#;

/// LLM-based Narsese translator using GPT-4o-mini
pub struct LlmNarseseTranslator {
    client: Client<OpenAIConfig>,
    model: String,
}

impl LlmNarseseTranslator {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            model: "gpt-4o-mini".to_string(),
        }
    }

    pub fn with_config(config: OpenAIConfig) -> Self {
        Self {
            client: Client::with_config(config),
            model: "gpt-4o-mini".to_string(),
        }
    }

    /// Translate natural language to Narsese statements
    /// Uses temperature 0.3 for deterministic output
    pub async fn nl_to_narsese(&self, query: &str) -> anyhow::Result<Vec<NarseseStatement>> {
        info!("Translating NL to Narsese: {}", query);

        let messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(NL_TO_NARSESE_PROMPT)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!("Translate this UI request to Narsese:\n{}", query))
                .build()?
                .into(),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .temperature(0.3) // Low temperature for deterministic output
            .max_tokens(1024u32)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        debug!("LLM Narsese output: {}", content);

        // Parse the response into statements
        let statements = self.parse_narsese_response(&content);
        info!("Parsed {} Narsese statements", statements.len());

        Ok(statements)
    }

    /// Translate Narsese statements to natural language
    /// Uses temperature 0.5 for more natural output
    pub async fn narsese_to_nl(&self, statements: &[NarseseStatement]) -> anyhow::Result<String> {
        if statements.is_empty() {
            return Ok(String::new());
        }

        let narsese_input: String = statements
            .iter()
            .map(|s| s.to_narsese())
            .collect::<Vec<_>>()
            .join("\n");

        info!("Translating {} Narsese statements to NL", statements.len());

        let messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(NARSESE_TO_NL_PROMPT)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!(
                    "Explain these Narsese statements in natural language:\n{}",
                    narsese_input
                ))
                .build()?
                .into(),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .temperature(0.5) // Moderate temperature for natural explanations
            .max_tokens(1024u32)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(content)
    }

    /// Parse LLM response into Narsese statements
    fn parse_narsese_response(&self, response: &str) -> Vec<NarseseStatement> {
        let mut statements = Vec::new();

        // Pattern: <...>. {freq|conf} or <...>. or just statements
        let re = regex::Regex::new(
            r"(<[^>]+>(?:\s*\.)?)\s*(?:\{([\d.]+)\|([\d.]+)\})?",
        )
        .unwrap();

        for line in response.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with("#") {
                continue;
            }

            if let Some(cap) = re.captures(line) {
                let statement = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                let frequency: f32 = cap
                    .get(2)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(1.0);
                let confidence: f32 = cap
                    .get(3)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0.9);

                if !statement.is_empty() {
                    // Clean up statement (remove trailing period if present)
                    let clean_stmt = statement.trim_end_matches('.');
                    statements.push(NarseseStatement::new(clean_stmt, frequency, confidence));
                }
            }
        }

        statements
    }
}

impl Default for LlmNarseseTranslator {
    fn default() -> Self {
        Self::new()
    }
}
