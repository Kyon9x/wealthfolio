//! Tauri-side implementation of AiEnvironment.
//!
//! Provides the wealthvn-ai crate with access to Tauri services
//! for tool execution and settings management.

use std::sync::{Arc, RwLock};

use wealthvn_ai::{AiEnvironment, SecretStore};
use wealthvn_core::{
    accounts::AccountServiceTrait,
    activities::ActivityServiceTrait,
    goals::GoalServiceTrait,
    holdings::HoldingsServiceTrait,
    income::IncomeServiceTrait,
    market_data::MarketDataServiceTrait,
    performance::PerformanceServiceTrait,
    settings::SettingsServiceTrait,
    valuation::ValuationServiceTrait,
};

use crate::ai_chat::ChatRepository;
use crate::secret_store::KeyringSecretStore;

/// Adapter to convert Tauri SecretStore to ai SecretStore.
pub struct SecretStoreAdapter(pub Arc<KeyringSecretStore>);

impl SecretStore for SecretStoreAdapter {
    fn set_secret(&self, service: &str, secret: &str) -> Result<(), wealthvn_ai::SecretStoreError> {
        self.0.set_secret(service, secret)
    }

    fn get_secret(&self, service: &str) -> Result<Option<String>, wealthvn_ai::SecretStoreError> {
        self.0.get_secret(service)
    }

    fn delete_secret(&self, service: &str) -> Result<(), wealthvn_ai::SecretStoreError> {
        self.0.delete_secret(service)
    }
}

/// Tauri-side implementation of AiEnvironment.
///
/// Wraps existing services from ServiceContext to provide access
/// to the AI crate for tool execution.
pub struct TauriAiEnvironment {
    base_currency: Arc<RwLock<String>>,
    account_service: Arc<dyn AccountServiceTrait + Send + Sync>,
    activity_service: Arc<dyn ActivityServiceTrait + Send + Sync>,
    holdings_service: Arc<dyn HoldingsServiceTrait + Send + Sync>,
    valuation_service: Arc<dyn ValuationServiceTrait + Send + Sync>,
    goal_service: Arc<dyn GoalServiceTrait + Send + Sync>,
    settings_service: Arc<dyn SettingsServiceTrait + Send + Sync>,
    secret_store: Arc<KeyringSecretStore>,
    chat_repository: Arc<ChatRepository>,
    market_data_service: Arc<dyn MarketDataServiceTrait + Send + Sync>,
    performance_service: Arc<dyn PerformanceServiceTrait + Send + Sync>,
    income_service: Arc<dyn IncomeServiceTrait + Send + Sync>,
}

impl TauriAiEnvironment {
    /// Create a new Tauri AI environment.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base_currency: Arc<RwLock<String>>,
        account_service: Arc<dyn AccountServiceTrait + Send + Sync>,
        activity_service: Arc<dyn ActivityServiceTrait + Send + Sync>,
        holdings_service: Arc<dyn HoldingsServiceTrait + Send + Sync>,
        valuation_service: Arc<dyn ValuationServiceTrait + Send + Sync>,
        goal_service: Arc<dyn GoalServiceTrait + Send + Sync>,
        settings_service: Arc<dyn SettingsServiceTrait + Send + Sync>,
        secret_store: Arc<KeyringSecretStore>,
        chat_repository: Arc<ChatRepository>,
        market_data_service: Arc<dyn MarketDataServiceTrait + Send + Sync>,
        performance_service: Arc<dyn PerformanceServiceTrait + Send + Sync>,
        income_service: Arc<dyn IncomeServiceTrait + Send + Sync>,
    ) -> Self {
        Self {
            base_currency,
            account_service,
            activity_service,
            holdings_service,
            valuation_service,
            goal_service,
            settings_service,
            secret_store,
            chat_repository,
            market_data_service,
            performance_service,
            income_service,
        }
    }
}

impl AiEnvironment for TauriAiEnvironment {
    fn base_currency(&self) -> String {
        self.base_currency.read().unwrap().clone()
    }

    fn account_service(&self) -> Arc<dyn AccountServiceTrait> {
        self.account_service.clone()
    }

    fn activity_service(&self) -> Arc<dyn ActivityServiceTrait> {
        self.activity_service.clone()
    }

    fn holdings_service(&self) -> Arc<dyn HoldingsServiceTrait> {
        self.holdings_service.clone()
    }

    fn valuation_service(&self) -> Arc<dyn ValuationServiceTrait> {
        self.valuation_service.clone()
    }

    fn goal_service(&self) -> Arc<dyn GoalServiceTrait> {
        self.goal_service.clone()
    }

    fn settings_service(&self) -> Arc<dyn SettingsServiceTrait> {
        self.settings_service.clone()
    }

    fn secret_store(&self) -> Arc<dyn SecretStore> {
        Arc::new(SecretStoreAdapter(self.secret_store.clone()))
    }

    fn chat_repository(&self) -> Arc<dyn wealthvn_ai::ChatRepositoryTrait> {
        self.chat_repository.clone()
    }

    fn market_data_service(&self) -> Arc<dyn MarketDataServiceTrait> {
        self.market_data_service.clone()
    }

    fn performance_service(&self) -> Arc<dyn PerformanceServiceTrait> {
        self.performance_service.clone()
    }

    fn income_service(&self) -> Arc<dyn IncomeServiceTrait> {
        self.income_service.clone()
    }
}
