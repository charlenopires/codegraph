//! VanillaCodeGenerator - LLM-powered code generation with graph context

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use tracing::{debug, info};

use crate::parser::{CodeParser, ParsedCode};
use crate::prompt::{self, SimilarElement, TemplateContext, SYSTEM_PROMPT};
use crate::templates::TemplateEngine;

/// Result of code generation
#[derive(Debug)]
pub struct GenerationResult {
    pub code: ParsedCode,
    pub html_document: String,
    pub validation_errors: Vec<String>,
}

/// Request for code generation
#[derive(Debug, Clone)]
pub struct GenerationRequest {
    pub description: String,
    pub similar_elements: Vec<SimilarElement>,
    pub reasoning: Option<String>,
    pub categories: Vec<String>,
}

/// VanillaCodeGenerator - generates HTML5/CSS3/ES6+ code using GPT-4o
pub struct VanillaCodeGenerator {
    client: Client<OpenAIConfig>,
    parser: CodeParser,
    template_engine: TemplateEngine,
    model: String,
}

impl VanillaCodeGenerator {
    /// Create a new generator with default OpenAI config
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            parser: CodeParser::new(),
            template_engine: TemplateEngine::new(),
            model: "gpt-4o".to_string(),
        }
    }

    /// Create with custom OpenAI config
    pub fn with_config(config: OpenAIConfig) -> Self {
        Self {
            client: Client::with_config(config),
            parser: CodeParser::new(),
            template_engine: TemplateEngine::new(),
            model: "gpt-4o".to_string(),
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Generate code from a request
    pub async fn generate(&self, request: GenerationRequest) -> anyhow::Result<GenerationResult> {
        info!("Generating code for: {}", request.description);

        // Get relevant templates based on categories
        let categories: Vec<&str> = request.categories.iter().map(|s| s.as_str()).collect();
        let templates: Vec<TemplateContext> = self
            .template_engine
            .get_matching(&categories)
            .into_iter()
            .map(|t| TemplateContext {
                name: t.name.clone(),
                html: t.html.clone(),
            })
            .collect();

        // Build the user prompt with context
        let user_prompt = prompt::build_user_prompt(
            &request.description,
            &request.similar_elements,
            &templates,
            request.reasoning.as_deref(),
        );

        debug!("User prompt: {}", user_prompt);

        // Call LLM
        let response = self.call_llm(&user_prompt).await?;

        debug!("LLM response length: {} chars", response.len());

        // Parse code blocks
        let code = self.parser.parse(&response);

        if code.is_empty() {
            anyhow::bail!("No code blocks found in LLM response");
        }

        // Validate HTML
        let validation_errors = if let Some(ref html) = code.html {
            match self.parser.validate_html(html) {
                Ok(()) => vec![],
                Err(errors) => errors,
            }
        } else {
            vec!["No HTML generated".to_string()]
        };

        // Generate full HTML document
        let html_document = code.to_html_document();

        Ok(GenerationResult {
            code,
            html_document,
            validation_errors,
        })
    }

    /// Call the LLM with system and user prompts
    async fn call_llm(&self, user_prompt: &str) -> anyhow::Result<String> {
        let messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(SYSTEM_PROMPT)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_prompt)
                .build()?
                .into(),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .temperature(0.7)
            .max_tokens(4096u32)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response from LLM"))?;

        Ok(content)
    }

    /// Get access to the template engine
    pub fn templates(&self) -> &TemplateEngine {
        &self.template_engine
    }
}

impl Default for VanillaCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
