//! Environment abstraction for AI assistant.
//!
//! This module provides the `AiEnvironment` trait that abstracts runtime
//! dependencies like secret stores, services, and configuration. The Tauri
//! backend implements this trait with its specific service instances.
//!
//! Note: wealth-vn currently only supports Tauri desktop. A server-side
//! implementation (Axum) can be added later by implementing this trait.

use async_trait::async_trait;
use std::sync::{Arc, RwLock};

use wealthvn_core::{
    accounts::AccountServiceTrait,
    activities::ActivityServiceTrait,
    goals::GoalServiceTrait,
    portfolio::{
        holdings::HoldingsServiceTrait,
        income::IncomeServiceTrait,
        performance::PerformanceServiceTrait,
        valuation::ValuationServiceTrait,
    },
    market_data::MarketDataServiceTrait,
    settings::SettingsServiceTrait,
};

use crate::types::ChatRepositoryTrait;

/// Environment abstraction for the AI assistant.
///
/// Implementations provide access to:
/// - Service traits for portfolio data access
/// - Secret store for API keys
/// - Configuration (base currency, etc.)
/// - Chat repository for thread/message persistence
/// - Market data service for symbol search
#[async_trait]
pub trait AiEnvironment: Send + Sync {
    /// Get the user's base currency (e.g., "USD", "VND").
    fn base_currency(&self) -> String;

    /// Get the account service for fetching accounts.
    fn account_service(&self) -> Arc<dyn AccountServiceTrait>;

    /// Get the activity service for fetching/saving activities.
    fn activity_service(&self) -> Arc<dyn ActivityServiceTrait>;

    /// Get the holdings service for fetching holdings.
    fn holdings_service(&self) -> Arc<dyn HoldingsServiceTrait>;

    /// Get the valuation service for fetching valuations.
    fn valuation_service(&self) -> Arc<dyn ValuationServiceTrait>;

    /// Get the goal service for fetching goals.
    fn goal_service(&self) -> Arc<dyn GoalServiceTrait>;

    /// Get the settings service for storing AI settings.
    fn settings_service(&self) -> Arc<dyn SettingsServiceTrait>;

    /// Get the secret store for API keys.
    fn secret_store(&self) -> Arc<dyn SecretStore>;

    /// Get the chat repository for thread/message persistence.
    fn chat_repository(&self) -> Arc<dyn ChatRepositoryTrait>;

    /// Get the market data service for symbol search.
    fn market_data_service(&self) -> Arc<dyn MarketDataServiceTrait>;

    /// Get the performance service for portfolio performance metrics.
    fn performance_service(&self) -> Arc<dyn PerformanceServiceTrait>;

    /// Get the income service for income/dividend summaries.
    fn income_service(&self) -> Arc<dyn IncomeServiceTrait>;
}

/// Secret store trait for API key management.
///
/// This trait is defined here to avoid a circular dependency
/// between src-ai and src-core/secrets.
pub trait SecretStore: Send + Sync {
    /// Store a secret for the given service.
    fn set_secret(&self, service: &str, secret: &str) -> Result<(), SecretStoreError>;

    /// Retrieve a secret for the given service.
    fn get_secret(&self, service: &str) -> Result<Option<String>, SecretStoreError>;

    /// Delete a secret for the given service.
    fn delete_secret(&self, service: &str) -> Result<(), SecretStoreError>;
}

/// Errors that can occur when interacting with the secret store.
#[derive(Debug, thiserror::Error)]
pub enum SecretStoreError {
    #[error("Failed to access secret store: {0}")]
    AccessFailed(String),

    #[error("Secret not found")]
    NotFound,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

impl From<keyring::Error> for SecretStoreError {
    fn from(err: keyring::Error) -> Self {
        match err {
            keyring::Error::NoEntry => SecretStoreError::NotFound,
            _ => SecretStoreError::AccessFailed(err.to_string()),
        }
    }
}

/// Tauri-side implementation of AiEnvironment.
///
/// Wraps existing services from ServiceContext to provide access
/// to the AI crate for tool execution.
pub struct TauriAiEnvironment {
    base_currency: Arc<RwLock<String>>,
    account_service: Arc<dyn AccountServiceTrait>,
    activity_service: Arc<dyn ActivityServiceTrait>,
    holdings_service: Arc<dyn HoldingsServiceTrait>,
    valuation_service: Arc<dyn ValuationServiceTrait>,
    goal_service: Arc<dyn GoalServiceTrait>,
    settings_service: Arc<dyn SettingsServiceTrait>,
    secret_store: Arc<dyn SecretStore>,
    chat_repository: Arc<dyn ChatRepositoryTrait>,
    market_data_service: Arc<dyn MarketDataServiceTrait>,
    performance_service: Arc<dyn PerformanceServiceTrait>,
    income_service: Arc<dyn IncomeServiceTrait>,
}

impl TauriAiEnvironment {
    /// Create a new Tauri AI environment.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base_currency: Arc<RwLock<String>>,
        account_service: Arc<dyn AccountServiceTrait>,
        activity_service: Arc<dyn ActivityServiceTrait>,
        holdings_service: Arc<dyn HoldingsServiceTrait>,
        valuation_service: Arc<dyn ValuationServiceTrait>,
        goal_service: Arc<dyn GoalServiceTrait>,
        settings_service: Arc<dyn SettingsServiceTrait>,
        secret_store: Arc<dyn SecretStore>,
        chat_repository: Arc<dyn ChatRepositoryTrait>,
        market_data_service: Arc<dyn MarketDataServiceTrait>,
        performance_service: Arc<dyn PerformanceServiceTrait>,
        income_service: Arc<dyn IncomeServiceTrait>,
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
        self.secret_store.clone()
    }

    fn chat_repository(&self) -> Arc<dyn ChatRepositoryTrait> {
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

#[cfg(test)]
pub mod test_env {
    use super::*;
    use crate::types::{ChatMessage, ChatThread, ListThreadsRequest, ThreadPage};
    use chrono::{DateTime, Utc};
    use std::collections::{HashMap, HashSet};
    use std::sync::RwLock;
    use wealthvn_core::{
        accounts::{Account, NewAccount},
        activities::{Activity, ActivityBulkMutationRequest, ActivityBulkMutationResult, ActivityImport, ActivitySearchResponse, ActivityUpdate, ImportMappingData, Sort},
        assets::Asset,
        errors::Result as CoreResult,
        goals::{Goal, GoalsAllocation, NewGoal},
        holdings::Holding,
        income::IncomeSummary,
        market_data::{AssetProfile, LatestQuotePair, MarketDataProviderInfo, Quote, QuoteSummary},
        performance::PerformanceMetrics,
        valuation::DailyAccountValuation,
        Error as CoreError,
    };

    /// Mock secret store for testing.
    #[derive(Default)]
    pub struct MockSecretStore {
        secrets: RwLock<HashMap<String, String>>,
    }

    impl SecretStore for MockSecretStore {
        fn set_secret(&self, key: &str, value: &str) -> Result<(), SecretStoreError> {
            self.secrets
                .write()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn get_secret(&self, key: &str) -> Result<Option<String>, SecretStoreError> {
            Ok(self.secrets.read().unwrap().get(key).cloned())
        }

        fn delete_secret(&self, key: &str) -> Result<(), SecretStoreError> {
            self.secrets.write().unwrap().remove(key);
            Ok(())
        }
    }

    /// Mock account service for testing.
    #[derive(Default)]
    pub struct MockAccountService {
        pub accounts: Vec<Account>,
    }

    #[async_trait]
    impl AccountServiceTrait for MockAccountService {
        async fn create_account(&self, _new_account: NewAccount) -> CoreResult<Account> {
            unimplemented!("MockAccountService::create_account")
        }

        async fn update_account(&self, _account_update: wealthvn_core::accounts::AccountUpdate) -> CoreResult<Account> {
            unimplemented!("MockAccountService::update_account")
        }

        async fn delete_account(&self, _account_id: &str) -> CoreResult<()> {
            unimplemented!("MockAccountService::delete_account")
        }

        fn get_account(&self, _account_id: &str) -> CoreResult<Account> {
            self.accounts.first().cloned().ok_or_else(|| {
                CoreError::Repository("No accounts found".to_string())
            })
        }

        fn list_accounts(
            &self,
            _is_active_filter: Option<bool>,
            _account_ids: Option<&[String]>,
        ) -> CoreResult<Vec<Account>> {
            Ok(self.accounts.clone())
        }

        fn get_all_accounts(&self) -> CoreResult<Vec<Account>> {
            Ok(self.accounts.clone())
        }

        fn get_active_accounts(&self) -> CoreResult<Vec<Account>> {
            Ok(self.accounts.clone())
        }

        fn get_accounts_by_ids(&self, _account_ids: &[String]) -> CoreResult<Vec<Account>> {
            Ok(self.accounts.clone())
        }
    }

    /// Mock activity service for testing.
    #[derive(Default)]
    pub struct MockActivityService;

    #[async_trait]
    impl ActivityServiceTrait for MockActivityService {
        fn get_activity(&self, _activity_id: &str) -> CoreResult<Activity> {
            unimplemented!("MockActivityService::get_activity")
        }

        fn get_activities(&self) -> CoreResult<Vec<Activity>> {
            Ok(Vec::new())
        }

        fn get_activities_by_account_id(&self, _account_id: &String) -> CoreResult<Vec<Activity>> {
            Ok(Vec::new())
        }

        fn get_activities_by_account_ids(&self, _account_ids: &[String]) -> CoreResult<Vec<Activity>> {
            Ok(Vec::new())
        }

        fn get_trading_activities(&self) -> CoreResult<Vec<Activity>> {
            Ok(Vec::new())
        }

        fn get_income_activities(&self) -> CoreResult<Vec<Activity>> {
            Ok(Vec::new())
        }

        fn search_activities(
            &self,
            _page: i64,
            _page_size: i64,
            _account_id_filter: Option<Vec<String>>,
            _activity_type_filter: Option<Vec<String>>,
            _asset_id_keyword: Option<String>,
            _sort: Option<Sort>,
        ) -> CoreResult<ActivitySearchResponse> {
            Ok(ActivitySearchResponse {
                data: Vec::new(),
                total_row_count: 0,
            })
        }

        fn get_first_activity_date(&self, _account_ids: Option<&[String]>) -> CoreResult<Option<DateTime<Utc>>> {
            Ok(None)
        }

        fn get_import_mapping(&self, _account_id: String) -> CoreResult<ImportMappingData> {
            Ok(ImportMappingData::default())
        }

        async fn create_activity(&self, _activity: wealthvn_core::activities::NewActivity) -> CoreResult<Activity> {
            unimplemented!("MockActivityService::create_activity")
        }

        async fn update_activity(&self, _activity: ActivityUpdate) -> CoreResult<Activity> {
            unimplemented!("MockActivityService::update_activity")
        }

        async fn delete_activity(&self, _activity_id: String) -> CoreResult<Activity> {
            unimplemented!("MockActivityService::delete_activity")
        }

        async fn bulk_mutate_activities(&self, _request: ActivityBulkMutationRequest) -> CoreResult<ActivityBulkMutationResult> {
            unimplemented!("MockActivityService::bulk_mutate_activities")
        }

        async fn check_activities_import(&self, _account_id: String, _activities: Vec<ActivityImport>) -> CoreResult<Vec<ActivityImport>> {
            unimplemented!("MockActivityService::check_activities_import")
        }

        async fn import_activities(&self, _account_id: String, _activities: Vec<ActivityImport>) -> CoreResult<Vec<ActivityImport>> {
            unimplemented!("MockActivityService::import_activities")
        }

        async fn save_import_mapping(&self, _mapping_data: ImportMappingData) -> CoreResult<ImportMappingData> {
            unimplemented!("MockActivityService::save_import_mapping")
        }
    }

    /// Mock holdings service for testing.
    #[derive(Default)]
    pub struct MockHoldingsService;

    #[async_trait]
    impl HoldingsServiceTrait for MockHoldingsService {
        async fn get_holdings(&self, _account_id: &str, _base_currency: &str) -> CoreResult<Vec<Holding>> {
            Ok(Vec::new())
        }

        async fn get_holding(&self, _account_id: &str, _asset_id: &str, _base_currency: &str) -> CoreResult<Option<Holding>> {
            Ok(None)
        }
    }

    /// Mock valuation service for testing.
    #[derive(Default)]
    pub struct MockValuationService;

    #[async_trait]
    impl ValuationServiceTrait for MockValuationService {
        async fn calculate_valuation_history(&self, _account_id: &str, _recalculate_all: bool) -> CoreResult<()> {
            Ok(())
        }

        fn get_historical_valuations(
            &self,
            _account_id: &str,
            _start_date_opt: Option<chrono::NaiveDate>,
            _end_date_opt: Option<chrono::NaiveDate>,
        ) -> CoreResult<Vec<DailyAccountValuation>> {
            Ok(Vec::new())
        }

        fn get_latest_valuations(&self, _account_ids: &[String]) -> CoreResult<Vec<DailyAccountValuation>> {
            Ok(Vec::new())
        }

        fn get_valuations_on_date(&self, _account_ids: &[String], _date: chrono::NaiveDate) -> CoreResult<Vec<DailyAccountValuation>> {
            Ok(Vec::new())
        }
    }

    /// Mock goal service for testing.
    #[derive(Default)]
    pub struct MockGoalService {
        pub goals: Vec<Goal>,
        pub allocations: Vec<GoalsAllocation>,
    }

    #[async_trait]
    impl GoalServiceTrait for MockGoalService {
        fn get_goals(&self) -> CoreResult<Vec<Goal>> {
            Ok(self.goals.clone())
        }

        async fn create_goal(&self, _new_goal: NewGoal) -> CoreResult<Goal> {
            unimplemented!("MockGoalService::create_goal")
        }

        async fn update_goal(&self, _updated_goal_data: Goal) -> CoreResult<Goal> {
            unimplemented!("MockGoalService::update_goal")
        }

        async fn delete_goal(&self, _goal_id_to_delete: String) -> CoreResult<usize> {
            unimplemented!("MockGoalService::delete_goal")
        }

        async fn upsert_goal_allocations(&self, _allocations: Vec<GoalsAllocation>) -> CoreResult<usize> {
            unimplemented!("MockGoalService::upsert_goal_allocations")
        }

        fn load_goals_allocations(&self) -> CoreResult<Vec<GoalsAllocation>> {
            Ok(self.allocations.clone())
        }

        fn validate_allocation_conflicts(
            &self,
            _account_id: &str,
            _start_date: &str,
            _end_date: &str,
            _percent_allocation: i32,
            _exclude_allocation_id: Option<&str>,
        ) -> CoreResult<()> {
            Ok(())
        }

        fn get_unallocated_balance(&self, _account_id: &str, _current_account_value: f64) -> CoreResult<f64> {
            Ok(0.0)
        }

        fn validate_unallocated_balance(&self, _account_id: &str, _allocation_amount: f64, _current_account_value: f64) -> CoreResult<()> {
            Ok(())
        }

        fn validate_allocation_percentages(&self, _account_id: &str, _new_percentage: f64, _exclude_allocation_id: Option<&str>) -> CoreResult<()> {
            Ok(())
        }

        fn get_repository(&self) -> &dyn wealthvn_core::goals::GoalRepositoryTrait {
            unimplemented!("MockGoalService::get_repository")
        }
    }

    /// Mock settings service for testing.
    #[derive(Default)]
    pub struct MockSettingsService;

    #[async_trait]
    impl SettingsServiceTrait for MockSettingsService {
        fn get_settings(&self) -> CoreResult<wealthvn_core::settings::Settings> {
            Ok(wealthvn_core::settings::Settings::default())
        }

        async fn update_settings(&self, _new_settings: &wealthvn_core::settings::SettingsUpdate) -> CoreResult<()> {
            Ok(())
        }

        fn get_base_currency(&self) -> CoreResult<Option<String>> {
            Ok(Some("USD".to_string()))
        }

        async fn update_base_currency(&self, _new_base_currency: &str) -> CoreResult<()> {
            Ok(())
        }

        fn is_auto_update_check_enabled(&self) -> CoreResult<bool> {
            Ok(true)
        }

        fn is_sync_enabled(&self) -> CoreResult<bool> {
            Ok(false)
        }
    }

    /// Mock chat repository for testing.
    #[derive(Default)]
    pub struct MockChatRepository {
        pub threads: RwLock<HashMap<String, ChatThread>>,
        pub messages: RwLock<HashMap<String, Vec<ChatMessage>>>,
    }

    #[async_trait]
    impl ChatRepositoryTrait for MockChatRepository {
        async fn create_thread(&self, thread: ChatThread) -> crate::types::ChatRepositoryResult<ChatThread> {
            self.threads
                .write()
                .unwrap()
                .insert(thread.id.clone(), thread.clone());
            Ok(thread)
        }

        fn get_thread(&self, thread_id: &str) -> crate::types::ChatRepositoryResult<Option<ChatThread>> {
            Ok(self.threads.read().unwrap().get(thread_id).cloned())
        }

        fn list_threads(&self, limit: i64, _offset: i64) -> crate::types::ChatRepositoryResult<Vec<ChatThread>> {
            let threads = self.threads.read().unwrap();
            let mut list: Vec<_> = threads.values().cloned().collect();
            list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            list.truncate(limit as usize);
            Ok(list)
        }

        fn list_threads_paginated(&self, request: &ListThreadsRequest) -> crate::types::ChatRepositoryResult<ThreadPage> {
            let threads = self.threads.read().unwrap();
            let mut list: Vec<_> = threads.values().cloned().collect();
            list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

            if let Some(ref search) = request.search {
                let search_lower = search.to_lowercase();
                list.retain(|t| {
                    t.title
                        .as_ref()
                        .map(|title| title.to_lowercase().contains(&search_lower))
                        .unwrap_or(false)
                });
            }

            let limit = request.limit.unwrap_or(20).min(100) as usize;
            let has_more = list.len() > limit;
            list.truncate(limit);

            let next_cursor = if has_more {
                list.last().map(|t| t.id.clone())
            } else {
                None
            };

            Ok(ThreadPage {
                threads: list,
                next_cursor,
                has_more,
            })
        }

        async fn update_thread(&self, thread: ChatThread) -> crate::types::ChatRepositoryResult<ChatThread> {
            self.threads
                .write()
                .unwrap()
                .insert(thread.id.clone(), thread.clone());
            Ok(thread)
        }

        async fn delete_thread(&self, thread_id: &str) -> crate::types::ChatRepositoryResult<()> {
            self.threads.write().unwrap().remove(thread_id);
            self.messages.write().unwrap().remove(thread_id);
            Ok(())
        }

        async fn create_message(&self, message: ChatMessage) -> crate::types::ChatRepositoryResult<ChatMessage> {
            self.messages
                .write()
                .unwrap()
                .entry(message.thread_id.clone())
                .or_default()
                .push(message.clone());
            Ok(message)
        }

        fn get_message(&self, message_id: &str) -> crate::types::ChatRepositoryResult<Option<ChatMessage>> {
            let messages = self.messages.read().unwrap();
            for msgs in messages.values() {
                if let Some(msg) = msgs.iter().find(|m| m.id == message_id) {
                    return Ok(Some(msg.clone()));
                }
            }
            Ok(None)
        }

        fn get_messages_by_thread(&self, thread_id: &str) -> crate::types::ChatRepositoryResult<Vec<ChatMessage>> {
            Ok(self
                .messages
                .read()
                .unwrap()
                .get(thread_id)
                .cloned()
                .unwrap_or_default())
        }

        async fn update_message(&self, message: ChatMessage) -> crate::types::ChatRepositoryResult<ChatMessage> {
            let mut messages = self.messages.write().unwrap();
            if let Some(msgs) = messages.get_mut(&message.thread_id) {
                if let Some(pos) = msgs.iter().position(|m| m.id == message.id) {
                    msgs[pos] = message.clone();
                }
            }
            Ok(message)
        }

        async fn add_tag(&self, _thread_id: &str, _tag: &str) -> crate::types::ChatRepositoryResult<()> {
            Ok(())
        }

        async fn remove_tag(&self, _thread_id: &str, _tag: &str) -> crate::types::ChatRepositoryResult<()> {
            Ok(())
        }

        fn get_tags(&self, _thread_id: &str) -> crate::types::ChatRepositoryResult<Vec<String>> {
            Ok(Vec::new())
        }
    }

    /// Mock market data service for testing.
    #[derive(Default)]
    pub struct MockMarketDataService;

    #[async_trait]
    impl MarketDataServiceTrait for MockMarketDataService {
        async fn search_symbol(&self, _query: &str) -> CoreResult<Vec<QuoteSummary>> {
            Ok(Vec::new())
        }

        fn get_latest_quote_for_symbol(&self, _symbol: &str) -> CoreResult<Quote> {
            unimplemented!("MockMarketDataService::get_latest_quote_for_symbol")
        }

        fn get_latest_quotes_for_symbols(&self, _symbols: &[String]) -> CoreResult<std::collections::HashMap<String, Quote>> {
            Ok(std::collections::HashMap::new())
        }

        fn get_all_historical_quotes(&self) -> CoreResult<std::collections::HashMap<String, Vec<(chrono::NaiveDate, Quote)>>> {
            Ok(std::collections::HashMap::new())
        }

        async fn get_asset_profile(&self, _symbol: &str) -> CoreResult<AssetProfile> {
            unimplemented!("MockMarketDataService::get_asset_profile")
        }

        fn get_historical_quotes_for_symbol(&self, _symbol: &str) -> CoreResult<Vec<Quote>> {
            Ok(Vec::new())
        }

        async fn add_quote(&self, _quote: &Quote) -> CoreResult<Quote> {
            unimplemented!("MockMarketDataService::add_quote")
        }

        async fn update_quote(&self, _quote: Quote) -> CoreResult<Quote> {
            unimplemented!("MockMarketDataService::update_quote")
        }

        async fn delete_quote(&self, _quote_id: &str) -> CoreResult<()> {
            unimplemented!("MockMarketDataService::delete_quote")
        }

        async fn get_historical_quotes_from_provider(
            &self,
            _symbol: &str,
            _start_date: chrono::NaiveDate,
            _end_date: chrono::NaiveDate,
        ) -> CoreResult<Vec<Quote>> {
            Ok(Vec::new())
        }

        async fn sync_market_data(&self) -> CoreResult<((), Vec<(String, String)>)> {
            Ok(((), Vec::new()))
        }

        async fn resync_market_data(&self, _symbols: Option<Vec<String>>) -> CoreResult<((), Vec<(String, String)>)> {
            Ok(((), Vec::new()))
        }

        fn get_latest_quotes_pair_for_symbols(&self, _symbols: &[String]) -> CoreResult<std::collections::HashMap<String, LatestQuotePair>> {
            Ok(std::collections::HashMap::new())
        }

        fn get_historical_quotes_for_symbols_in_range(
            &self,
            _symbols: &HashSet<String>,
            _start_date: chrono::NaiveDate,
            _end_date: chrono::NaiveDate,
        ) -> CoreResult<Vec<Quote>> {
            Ok(Vec::new())
        }

        async fn get_daily_quotes(
            &self,
            _asset_ids: &HashSet<String>,
            _start_date: chrono::NaiveDate,
            _end_date: chrono::NaiveDate,
        ) -> CoreResult<std::collections::HashMap<chrono::NaiveDate, std::collections::HashMap<String, Quote>>> {
            Ok(std::collections::HashMap::new())
        }

        async fn get_market_data_providers_info(&self) -> CoreResult<Vec<MarketDataProviderInfo>> {
            Ok(Vec::new())
        }

        async fn get_market_data_providers_settings(&self) -> CoreResult<Vec<wealthvn_core::market_data::MarketDataProviderSetting>> {
            Ok(Vec::new())
        }

        async fn update_market_data_provider_settings(
            &self,
            _provider_id: String,
            _priority: i32,
            _enabled: bool,
        ) -> CoreResult<wealthvn_core::market_data::MarketDataProviderSetting> {
            unimplemented!("MockMarketDataService::update_market_data_provider_settings")
        }

        async fn import_quotes_from_csv(
            &self,
            _quotes: Vec<wealthvn_core::market_data::QuoteImport>,
            _overwrite: bool,
        ) -> CoreResult<Vec<wealthvn_core::market_data::QuoteImport>> {
            unimplemented!("MockMarketDataService::import_quotes_from_csv")
        }

        async fn bulk_upsert_quotes(&self, _quotes: Vec<Quote>) -> CoreResult<usize> {
            unimplemented!("MockMarketDataService::bulk_upsert_quotes")
        }
    }

    /// Mock performance service for testing.
    #[derive(Default)]
    pub struct MockPerformanceService;

    #[async_trait]
    impl PerformanceServiceTrait for MockPerformanceService {
        async fn calculate_performance_history(
            &self,
            _item_type: &str,
            _item_id: &str,
            _start_date: Option<chrono::NaiveDate>,
            _end_date: Option<chrono::NaiveDate>,
        ) -> CoreResult<PerformanceMetrics> {
            Ok(PerformanceMetrics::default())
        }

        async fn calculate_performance_summary(
            &self,
            _item_type: &str,
            _item_id: &str,
            _start_date: Option<chrono::NaiveDate>,
            _end_date: Option<chrono::NaiveDate>,
        ) -> CoreResult<PerformanceMetrics> {
            Ok(PerformanceMetrics::default())
        }

        fn calculate_accounts_simple_performance(&self, _account_ids: &[String]) -> CoreResult<Vec<wealthvn_core::performance::SimplePerformanceMetrics>> {
            Ok(Vec::new())
        }
    }

    /// Mock income service for testing.
    #[derive(Default)]
    pub struct MockIncomeService;

    impl IncomeServiceTrait for MockIncomeService {
        fn get_income_summary(&self) -> CoreResult<Vec<IncomeSummary>> {
            Ok(vec![
                IncomeSummary::new("TOTAL", "USD".to_string()),
                IncomeSummary::new("YTD", "USD".to_string()),
                IncomeSummary::new("LAST_YEAR", "USD".to_string()),
            ])
        }
    }

    /// Mock environment for testing.
    pub struct MockEnvironment {
        pub base_currency: String,
        pub account_service: Arc<dyn AccountServiceTrait>,
        pub activity_service: Arc<dyn ActivityServiceTrait>,
        pub holdings_service: Arc<dyn HoldingsServiceTrait>,
        pub valuation_service: Arc<dyn ValuationServiceTrait>,
        pub goal_service: Arc<dyn GoalServiceTrait>,
        pub settings_service: Arc<dyn SettingsServiceTrait>,
        pub secret_store: Arc<dyn SecretStore>,
        pub chat_repository: Arc<dyn ChatRepositoryTrait>,
        pub market_data_service: Arc<dyn MarketDataServiceTrait>,
        pub performance_service: Arc<dyn PerformanceServiceTrait>,
        pub income_service: Arc<dyn IncomeServiceTrait>,
    }

    impl Default for MockEnvironment {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockEnvironment {
        pub fn new() -> Self {
            Self {
                base_currency: "USD".to_string(),
                account_service: Arc::new(MockAccountService::default()),
                activity_service: Arc::new(MockActivityService::default()),
                holdings_service: Arc::new(MockHoldingsService::default()),
                valuation_service: Arc::new(MockValuationService::default()),
                goal_service: Arc::new(MockGoalService::default()),
                settings_service: Arc::new(MockSettingsService::default()),
                secret_store: Arc::new(MockSecretStore::default()),
                chat_repository: Arc::new(MockChatRepository::default()),
                market_data_service: Arc::new(MockMarketDataService::default()),
                performance_service: Arc::new(MockPerformanceService::default()),
                income_service: Arc::new(MockIncomeService::default()),
            }
        }

        pub fn with_secret(self, key: &str, value: &str) -> Self {
            self.secret_store.set_secret(key, value).unwrap();
            self
        }
    }

    #[async_trait]
    impl AiEnvironment for MockEnvironment {
        fn base_currency(&self) -> String {
            self.base_currency.clone()
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
            self.secret_store.clone()
        }

        fn chat_repository(&self) -> Arc<dyn ChatRepositoryTrait> {
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
}
