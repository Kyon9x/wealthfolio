//! AI provider catalog and client management.
//!
//! This module provides:
//! - Provider catalog loaded from JSON configuration
//! - Client factory for rig-core providers
//! - API key management via the environment's secret store

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::env::{AiEnvironment, SecretStoreError};
use crate::error::AiError;
use crate::provider_model::{
    CapabilityInfo, ConnectionField, ModelCapabilities, ProviderDefaultConfig,
};

// ============================================================================
// Provider Catalog (Static JSON)
// ============================================================================

/// Static provider catalog loaded from embedded JSON.
static PROVIDER_CATALOG: Lazy<ProviderCatalog> = Lazy::new(|| {
    let json = include_str!("../ai_providers.json");
    serde_json::from_str(json).expect("Failed to parse ai_providers.json")
});

#[derive(Debug, Deserialize)]
struct ProviderCatalog {
    providers: HashMap<String, ProviderCatalogEntry>,
    capabilities: HashMap<String, CapabilityInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProviderCatalogEntry {
    name: String,
    #[serde(rename = "type")]
    provider_type: String,
    icon: String,
    description: String,
    #[serde(default)]
    env_key: Option<String>,
    #[serde(default)]
    default_config: ProviderDefaultConfig,
    #[serde(default)]
    connection_fields: Vec<ConnectionField>,
    models: HashMap<String, ModelCatalogEntry>,
    default_model: String,
    /// Fast model for title generation (falls back to default_model if not set).
    #[serde(default)]
    title_model_id: Option<String>,
    #[serde(default)]
    documentation_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelCatalogEntry {
    #[serde(default)]
    capabilities: ModelCapabilities,
}

// ============================================================================
// Local Types (simplified views for this service)
// ============================================================================

/// Simple provider info for catalog listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleProviderInfo {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub icon: String,
    pub description: String,
    pub default_model: String,
    pub documentation_url: Option<String>,
    #[serde(default)]
    pub default_config: ProviderDefaultConfig,
    #[serde(default)]
    pub connection_fields: Vec<ConnectionField>,
    #[serde(default)]
    pub models: Vec<SimpleModelInfo>,
    #[serde(default)]
    pub env_key: Option<String>,
}

/// Simple model info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleModelInfo {
    pub id: String,
    #[serde(default)]
    pub capabilities: ModelCapabilities,
}

/// Provider setting for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleProviderSetting {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider_type: String,
    pub icon: String,
    pub default_model: String,
    pub enabled: bool,
    #[serde(default)]
    pub supports_custom_url: bool,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub documentation_url: Option<String>,
    #[serde(default)]
    pub env_key: Option<String>,
    #[serde(default)]
    pub models: Vec<SimpleModelInfo>,
}

/// Combined settings response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleSettings {
    pub provider_id: String,
    pub model: String,
    pub has_api_key: bool,
    pub providers: Vec<SimpleProviderSetting>,
    #[serde(default)]
    pub capabilities: HashMap<String, CapabilityInfo>,
}

// ============================================================================
// Provider Service
// ============================================================================

/// Service key for storing AI provider settings.
pub const AI_SETTINGS_KEY: &str = "ai_settings";

/// Trait for AI provider service operations.
///
/// This trait abstracts the provider service to allow for different
/// implementations (e.g., in tests or mock environments).
pub trait AiProviderServiceTrait: Send + Sync {
    /// Get all provider info from the catalog.
    fn get_provider_catalog(&self) -> Vec<SimpleProviderInfo>;

    /// Get capability info.
    fn get_capabilities(&self) -> HashMap<String, CapabilityInfo>;

    /// Get the current AI settings.
    fn get_settings(&self) -> Result<SimpleSettings, AiError>;

    /// Get API key for a provider from the secret store.
    fn get_api_key(&self, provider_id: &str) -> Result<Option<String>, AiError>;

    /// Check if a provider has an API key stored.
    fn has_api_key(&self, provider_id: &str) -> bool;

    /// Set API key for a provider.
    fn set_api_key(&self, provider_id: &str, api_key: &str) -> Result<(), AiError>;

    /// Delete API key for a provider.
    fn delete_api_key(&self, provider_id: &str) -> Result<(), AiError>;

    /// Get model capabilities for a specific provider/model combination.
    fn get_model_capabilities(&self, provider_id: &str, model_id: &str) -> ModelCapabilities;

    /// Get the title model ID for a provider.
    fn get_title_model(&self, provider_id: &str) -> Option<String>;

    /// Get the tools allowlist for a provider.
    fn get_tools_allowlist(&self, provider_id: &str) -> Option<Vec<String>>;

    /// Get provider URL (for local providers like Ollama).
    fn get_provider_url(&self, provider_id: &str) -> Option<String>;

    /// Update AI settings.
    fn update_settings(
        &self,
        provider_id: Option<String>,
        model: Option<String>,
        provider_config: Option<StoredProviderConfig>,
    ) -> Result<SimpleSettings, AiError>;
}

/// Provider service for managing AI settings.
pub struct ProviderService<E: AiEnvironment> {
    env: Arc<E>,
}

impl<E: AiEnvironment> ProviderService<E> {
    /// Create a new provider service.
    pub fn new(env: Arc<E>) -> Self {
        Self { env }
    }

    /// Get all provider info from the catalog.
    pub fn get_provider_catalog(&self) -> Vec<SimpleProviderInfo> {
        PROVIDER_CATALOG
            .providers
            .iter()
            .map(|(id, entry)| SimpleProviderInfo {
                id: id.clone(),
                name: entry.name.clone(),
                provider_type: entry.provider_type.clone(),
                icon: entry.icon.clone(),
                description: entry.description.clone(),
                default_model: entry.default_model.clone(),
                documentation_url: entry.documentation_url.clone(),
                default_config: entry.default_config.clone(),
                connection_fields: entry.connection_fields.clone(),
                models: entry
                    .models
                    .iter()
                    .map(|(id, m)| SimpleModelInfo {
                        id: id.clone(),
                        capabilities: m.capabilities.clone(),
                    })
                    .collect(),
                env_key: entry.env_key.clone(),
            })
            .collect()
    }

    /// Get capability info.
    pub fn get_capabilities(&self) -> HashMap<String, CapabilityInfo> {
        PROVIDER_CATALOG.capabilities.clone()
    }

    /// Get the current AI settings (merged from catalog + stored settings).
    ///
    /// Note: This is a simplified version that returns catalog defaults.
    /// Full settings persistence will be implemented when SettingsServiceTrait
    /// is extended with get_setting_value/set_setting_value methods.
    pub fn get_settings(&self) -> Result<SimpleSettings, AiError> {
        // Use default provider and model from catalog
        let provider_id = "ollama".to_string();
        let model = PROVIDER_CATALOG
            .providers
            .get(&provider_id)
            .map(|p| p.default_model.clone())
            .unwrap_or_else(|| "deepseek-r1:8b".to_string());

        // Check if we have an API key
        let has_api_key = self.has_api_key(&provider_id);

        // Build provider settings from catalog defaults
        let providers: Vec<SimpleProviderSetting> = PROVIDER_CATALOG
            .providers
            .iter()
            .map(|(id, entry)| {
                let url = entry.default_config.url.clone();

                SimpleProviderSetting {
                    id: id.clone(),
                    name: entry.name.clone(),
                    description: entry.description.clone(),
                    provider_type: entry.provider_type.clone(),
                    icon: entry.icon.clone(),
                    default_model: entry.default_model.clone(),
                    enabled: entry.default_config.enabled,
                    supports_custom_url: entry.provider_type == "local",
                    url,
                    documentation_url: entry.documentation_url.clone(),
                    env_key: entry.env_key.clone(),
                    models: entry
                        .models
                        .iter()
                        .map(|(id, m)| SimpleModelInfo {
                            id: id.clone(),
                            capabilities: m.capabilities.clone(),
                        })
                        .collect(),
                }
            })
            .collect();

        Ok(SimpleSettings {
            provider_id,
            model,
            has_api_key,
            providers,
            capabilities: PROVIDER_CATALOG.capabilities.clone(),
        })
    }

    /// Build the secret key for a provider (format: ai_<provider_id>).
    /// Matches the frontend convention in use-ai-providers.ts.
    fn secret_key_for_provider(provider_id: &str) -> String {
        format!("ai_{}", provider_id)
    }

    /// Get API key for a provider from the secret store.
    pub fn get_api_key(&self, provider_id: &str) -> Result<Option<String>, AiError> {
        let secret_key = Self::secret_key_for_provider(provider_id);
        match self.env.secret_store().get_secret(&secret_key) {
            Ok(value) => Ok(value),
            Err(SecretStoreError::NotFound) => Ok(None),
            Err(e) => Err(AiError::Internal(e.to_string())),
        }
    }

    /// Check if a provider has an API key stored.
    pub fn has_api_key(&self, provider_id: &str) -> bool {
        self.get_api_key(provider_id)
            .ok()
            .flatten()
            .map(|k| !k.is_empty())
            .unwrap_or(false)
    }

    /// Set API key for a provider.
    pub fn set_api_key(&self, provider_id: &str, api_key: &str) -> Result<(), AiError> {
        let secret_key = Self::secret_key_for_provider(provider_id);
        self.env
            .secret_store()
            .set_secret(&secret_key, api_key)
            .map_err(|e| AiError::Internal(e.to_string()))
    }

    /// Delete API key for a provider.
    pub fn delete_api_key(&self, provider_id: &str) -> Result<(), AiError> {
        let secret_key = Self::secret_key_for_provider(provider_id);
        self.env
            .secret_store()
            .delete_secret(&secret_key)
            .map_err(|e| AiError::Internal(e.to_string()))
    }

    /// Get model capabilities for a specific provider/model combination.
    /// Returns catalog capabilities (user overrides not supported yet).
    pub fn get_model_capabilities(&self, provider_id: &str, model_id: &str) -> ModelCapabilities {
        // Get base capabilities from catalog
        PROVIDER_CATALOG
            .providers
            .get(provider_id)
            .and_then(|p| p.models.get(model_id))
            .map(|m| m.capabilities.clone())
            .unwrap_or(ModelCapabilities {
                tools: false,
                thinking: false,
                vision: false,
                streaming: true,
            })
    }

    /// Get the title model ID for a provider.
    /// Returns title_model_id if configured, otherwise falls back to default_model.
    pub fn get_title_model(&self, provider_id: &str) -> Option<String> {
        PROVIDER_CATALOG.providers.get(provider_id).map(|p| {
            p.title_model_id
                .clone()
                .unwrap_or_else(|| p.default_model.clone())
        })
    }

    /// Get the tools allowlist for a provider.
    /// Currently returns None (all tools enabled).
    pub fn get_tools_allowlist(&self, _provider_id: &str) -> Option<Vec<String>> {
        // TODO: Implement when settings persistence is available
        None
    }

    /// Get provider URL (for local providers like Ollama).
    /// Returns catalog default (user overrides not supported yet).
    pub fn get_provider_url(&self, provider_id: &str) -> Option<String> {
        PROVIDER_CATALOG
            .providers
            .get(provider_id)
            .and_then(|p| p.default_config.url.clone())
            .filter(|u| reqwest::Url::parse(u).is_ok())
    }

    /// Update AI settings.
    ///
    /// Note: This is a simplified version that doesn't persist settings.
    /// Full settings persistence will be implemented when SettingsServiceTrait
    /// is extended with get_setting_value/set_setting_value methods.
    pub fn update_settings(
        &self,
        _provider_id: Option<String>,
        _model: Option<String>,
        _provider_config: Option<StoredProviderConfig>,
    ) -> Result<SimpleSettings, AiError> {
        // Return current settings (updates not persisted yet)
        self.get_settings()
    }
}

impl<E: AiEnvironment> AiProviderServiceTrait for ProviderService<E> {
    fn get_provider_catalog(&self) -> Vec<SimpleProviderInfo> {
        self.get_provider_catalog()
    }

    fn get_capabilities(&self) -> HashMap<String, CapabilityInfo> {
        self.get_capabilities()
    }

    fn get_settings(&self) -> Result<SimpleSettings, AiError> {
        self.get_settings()
    }

    fn get_api_key(&self, provider_id: &str) -> Result<Option<String>, AiError> {
        self.get_api_key(provider_id)
    }

    fn has_api_key(&self, provider_id: &str) -> bool {
        self.has_api_key(provider_id)
    }

    fn set_api_key(&self, provider_id: &str, api_key: &str) -> Result<(), AiError> {
        self.set_api_key(provider_id, api_key)
    }

    fn delete_api_key(&self, provider_id: &str) -> Result<(), AiError> {
        self.delete_api_key(provider_id)
    }

    fn get_model_capabilities(&self, provider_id: &str, model_id: &str) -> ModelCapabilities {
        self.get_model_capabilities(provider_id, model_id)
    }

    fn get_title_model(&self, provider_id: &str) -> Option<String> {
        self.get_title_model(provider_id)
    }

    fn get_tools_allowlist(&self, provider_id: &str) -> Option<Vec<String>> {
        self.get_tools_allowlist(provider_id)
    }

    fn get_provider_url(&self, provider_id: &str) -> Option<String> {
        self.get_provider_url(provider_id)
    }

    fn update_settings(
        &self,
        provider_id: Option<String>,
        model: Option<String>,
        provider_config: Option<StoredProviderConfig>,
    ) -> Result<SimpleSettings, AiError> {
        self.update_settings(provider_id, model, provider_config)
    }
}

/// Stored AI settings (in app_settings).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredAiSettings {
    pub provider_id: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub providers: HashMap<String, StoredProviderSettings>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredProviderSettings {
    pub enabled: Option<bool>,
    pub url: Option<String>,
}

/// Config update for a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredProviderConfig {
    pub id: String,
    pub enabled: bool,
    pub url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_catalog_loads() {
        let catalog = &*PROVIDER_CATALOG;
        assert!(!catalog.providers.is_empty());
        assert!(catalog.providers.contains_key("openai"));
        assert!(catalog.providers.contains_key("ollama"));
    }

    #[test]
    fn test_capabilities_loads() {
        let catalog = &*PROVIDER_CATALOG;
        assert!(catalog.capabilities.contains_key("tools"));
        assert!(catalog.capabilities.contains_key("thinking"));
    }

    #[test]
    fn test_provider_catalog_has_all_providers() {
        let catalog = &*PROVIDER_CATALOG;
        // Verify all expected providers are present
        assert!(catalog.providers.contains_key("openai"));
        assert!(catalog.providers.contains_key("anthropic"));
        assert!(catalog.providers.contains_key("google"));
        assert!(catalog.providers.contains_key("groq"));
        assert!(catalog.providers.contains_key("ollama"));
        assert!(catalog.providers.contains_key("openrouter"));
    }

    #[test]
    fn test_provider_models_have_capabilities() {
        let catalog = &*PROVIDER_CATALOG;

        // Check OpenAI models
        let openai = catalog.providers.get("openai").unwrap();
        assert!(openai.models.contains_key("gpt-5-mini"));
        let gpt5_mini = openai.models.get("gpt-5-mini").unwrap();
        assert!(gpt5_mini.capabilities.tools);
        assert!(gpt5_mini.capabilities.vision);
        assert!(!gpt5_mini.capabilities.thinking);

        // Check Anthropic models
        let anthropic = catalog.providers.get("anthropic").unwrap();
        assert!(anthropic.models.contains_key("claude-sonnet-4-5-20250929"));
    }
}
