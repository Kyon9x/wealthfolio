//! Title generation service for chat threads.
//!
//! Auto-generates short descriptive titles from user messages using a fast model.
//! Falls back to truncating the first user message if generation fails.

use async_trait::async_trait;
use log::{debug, warn};
use reqwest::Client as HttpClient;
use rig::{
    client::{CompletionClient, Nothing},
    completion::Prompt,
    providers::{anthropic, gemini, groq, ollama, openai, openrouter},
};

use crate::env::AiEnvironment;
use crate::error::AiError;
use crate::providers::ProviderService;
use std::sync::Arc;

// ============================================================================
// Title Generator Trait
// ============================================================================

/// Trait for generating thread titles.
#[async_trait]
pub trait TitleGeneratorTrait: Send + Sync {
    /// Generate a title from the first user message.
    ///
    /// Returns a short (4 words max) descriptive title suitable for sidebar display.
    /// Falls back to truncating the message if LLM generation fails.
    ///
    /// `chat_model_id` is used as fallback if no title model is configured for the provider.
    async fn generate_title(
        &self,
        user_message: &str,
        provider_id: &str,
        chat_model_id: &str,
    ) -> String;
}

// ============================================================================
// Title Generator Implementation
// ============================================================================

/// Configuration for title generation.
pub struct TitleGeneratorConfig {
    /// Max characters for the truncated fallback title.
    pub fallback_max_chars: usize,
    /// Max tokens to generate for the title.
    pub max_tokens: u32,
    /// Temperature for title generation (lower = more focused).
    pub temperature: f32,
}

impl Default for TitleGeneratorConfig {
    fn default() -> Self {
        Self {
            fallback_max_chars: 50,
            max_tokens: 20,
            temperature: 0.3,
        }
    }
}

/// Title generator implementation using LLM providers.
pub struct TitleGenerator<E: AiEnvironment> {
    env: Arc<E>,
    config: TitleGeneratorConfig,
}

impl<E: AiEnvironment> TitleGenerator<E> {
    /// Create a new title generator.
    pub fn new(env: Arc<E>, config: TitleGeneratorConfig) -> Self {
        Self { env, config }
    }

    /// Generate a fallback title by truncating the user message.
    fn generate_fallback(&self, user_message: &str) -> String {
        truncate_to_title(user_message, self.config.fallback_max_chars)
    }

    /// Generate a title using the LLM with a specific model.
    async fn generate_with_llm(
        &self,
        user_message: &str,
        provider_id: &str,
        model_id: &str,
    ) -> Result<String, AiError> {
        let provider_service = ProviderService::new(self.env.clone());
        let api_key = provider_service.get_api_key(provider_id)?;
        let provider_url = provider_service.get_provider_url(provider_id);

        debug!(
            "Generating title with provider {} model {}",
            provider_id, model_id
        );

        let prompt = format!(
            "Generate a very short plain-text title (max 4 words) for this chat message.\n\
Rules:\n\
- Return ONLY the title text\n\
- No markdown (no **bold**, no *italics*, no backticks)\n\
- No quotes\n\
- No leading \"Title:\" prefix\n\n\
Message:\n\"{}\"\n\n\
Title:",
            truncate_to_title(user_message, 200)
        );

        let response = match provider_id {
            "anthropic" => {
                let key = api_key.ok_or_else(|| AiError::MissingApiKey(provider_id.to_string()))?;
                let mut builder = anthropic::Client::<HttpClient>::builder().api_key(&key);
                if let Some(url) = provider_url {
                    builder = builder.base_url(&url);
                }
                let client = builder
                    .build()
                    .map_err(|e| AiError::Provider(e.to_string()))?;
                client
                    .agent(model_id)
                    .max_tokens(self.config.max_tokens as u64)
                    .build()
                    .prompt(&prompt)
                    .await
                    .map_err(|e| AiError::Provider(e.to_string()))?
            }
            "gemini" | "google" => {
                let key = api_key.ok_or_else(|| AiError::MissingApiKey(provider_id.to_string()))?;
                let mut builder = gemini::Client::<HttpClient>::builder().api_key(&key);
                if let Some(url) = provider_url {
                    builder = builder.base_url(&url);
                }
                let client = builder
                    .build()
                    .map_err(|e| AiError::Provider(e.to_string()))?;
                client
                    .agent(model_id)
                    .build()
                    .prompt(&prompt)
                    .await
                    .map_err(|e| AiError::Provider(e.to_string()))?
            }
            "groq" => {
                let key = api_key.ok_or_else(|| AiError::MissingApiKey(provider_id.to_string()))?;
                let mut builder = groq::Client::<HttpClient>::builder().api_key(&key);
                if let Some(url) = provider_url {
                    builder = builder.base_url(&url);
                }
                let client = builder
                    .build()
                    .map_err(|e| AiError::Provider(e.to_string()))?;
                client
                    .agent(model_id)
                    .build()
                    .prompt(&prompt)
                    .await
                    .map_err(|e| AiError::Provider(e.to_string()))?
            }
            "ollama" => {
                let mut builder = ollama::Client::<HttpClient>::builder().api_key(Nothing);
                if let Some(url) = provider_url {
                    builder = builder.base_url(&url);
                }
                let client = builder
                    .build()
                    .map_err(|e| AiError::Provider(e.to_string()))?;
                client
                    .agent(model_id)
                    .build()
                    .prompt(&prompt)
                    .await
                    .map_err(|e| AiError::Provider(e.to_string()))?
            }
            "openrouter" => {
                let key = api_key.ok_or_else(|| AiError::MissingApiKey(provider_id.to_string()))?;
                let mut builder = openrouter::Client::<HttpClient>::builder().api_key(&key);
                if let Some(url) = provider_url {
                    builder = builder.base_url(&url);
                }
                let client = builder
                    .build()
                    .map_err(|e| AiError::Provider(e.to_string()))?;
                client
                    .agent(model_id)
                    .build()
                    .prompt(&prompt)
                    .await
                    .map_err(|e| AiError::Provider(e.to_string()))?
            }
            _ => {
                // Default to OpenAI-compatible
                let key = api_key.ok_or_else(|| AiError::MissingApiKey(provider_id.to_string()))?;
                let mut builder = openai::Client::<HttpClient>::builder().api_key(&key);
                if let Some(url) = provider_url {
                    builder = builder.base_url(&url);
                }
                let client = builder
                    .build()
                    .map_err(|e| AiError::Provider(e.to_string()))?;
                client
                    .agent(model_id)
                    .max_tokens(self.config.max_tokens as u64)
                    .build()
                    .prompt(&prompt)
                    .await
                    .map_err(|e| AiError::Provider(e.to_string()))?
            }
        };

        let title = response.to_string().trim().to_string();
        Ok(title)
    }
}

#[async_trait]
impl<E: AiEnvironment + 'static> TitleGeneratorTrait for TitleGenerator<E> {
    async fn generate_title(
        &self,
        user_message: &str,
        provider_id: &str,
        chat_model_id: &str,
    ) -> String {
        // Get the title model for this provider (fallback to chat model)
        let provider_service = ProviderService::new(self.env.clone());
        let model_id = provider_service
            .get_title_model(provider_id)
            .unwrap_or_else(|| chat_model_id.to_string());

        // Try LLM generation, fall back to truncation on any error
        match self
            .generate_with_llm(user_message, provider_id, &model_id)
            .await
        {
            Ok(title) if !title.trim().is_empty() => truncate_to_title(&title, 50),
            _ => {
                warn!("Title generation failed, using fallback");
                self.generate_fallback(user_message)
            }
        }
    }
}

// ============================================================================
// Title Truncation Helper
// ============================================================================

/// Truncate a message to a short title (first few words, max chars).
pub fn truncate_to_title(message: &str, max_chars: usize) -> String {
    let trimmed = message.trim().to_string();

    // Split into words and take first few
    let words: Vec<&str> = trimmed
        .split_whitespace()
        .take(5) // Max 5 words
        .collect();

    let title = if words.len() < trimmed.split_whitespace().count() {
        format!("{}...", words.join(" "))
    } else {
        words.join(" ")
    };

    // Truncate to max_chars if needed
    if title.len() > max_chars {
        if let Some(pos) = title.char_indices().nth(max_chars.saturating_sub(3)) {
            format!("{}...", &title[..pos.0])
        } else {
            title
        }
    } else {
        title
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_to_title() {
        assert_eq!(truncate_to_title("Hello world", 50), "Hello world");
        assert_eq!(
            truncate_to_title("This is a very long message that needs truncation", 20),
            "This is a very..."
        );
        assert_eq!(
            truncate_to_title("Hello world this is a test", 20),
            "Hello world..."
        );
    }

    #[test]
    fn test_truncate_empty() {
        assert_eq!(truncate_to_title("", 50), "");
        assert_eq!(truncate_to_title("   ", 50), "");
    }
}
