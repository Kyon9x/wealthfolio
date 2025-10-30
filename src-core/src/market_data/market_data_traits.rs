use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

use crate::errors::Result;
use crate::market_data::market_data_model::{
    MarketDataProviderSetting, UpdateMarketDataProviderSetting, QuoteRequest, ImportValidationStatus,
};
use super::market_data_model::{Quote, QuoteSummary, LatestQuotePair, MarketDataProviderInfo, QuoteDb, QuoteImport};
use super::providers::models::AssetProfile;

#[async_trait]
pub trait MarketDataServiceTrait: Send + Sync {
    async fn search_symbol(&self, query: &str) -> Result<Vec<QuoteSummary>>;
    
    // New async methods
    async fn get_latest_quote(
        &self,
        symbol: &str,
        currency: &str,
    ) -> Result<Option<Quote>>;
    
    async fn get_latest_quotes_bulk(
        &self,
        quote_requests: &[QuoteRequest],
    ) -> Result<Vec<Quote>>;
    
    async fn get_historical_quotes(
        &self,
        symbol: &str,
        start: SystemTime,
        end: SystemTime,
        currency: &str,
    ) -> Result<Vec<Quote>>;
    
    async fn get_historical_quotes_for_symbols_in_range(
        &self,
        symbols: &HashSet<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<Quote>>;
    
    async fn import_quotes(&self, quotes: Vec<QuoteImport>) -> Result<Vec<ImportValidationStatus>>;
    

    async fn get_asset_profile(&self, symbol: &str) -> Result<Option<AssetProfile>>;
    async fn validate_quote_import(&self, quote_import: &QuoteImport) -> ImportValidationStatus;
    
    async fn get_provider_settings(&self) -> Result<Vec<MarketDataProviderSetting>>;
    
    async fn update_provider_setting(
        &self,
        provider_id: String,
        update: UpdateMarketDataProviderSetting,
    ) -> Result<()>;
    
    async fn get_provider_info(&self) -> Result<Vec<MarketDataProviderInfo>>;
    
    async fn import_quotes_from_csv(&self, quotes: Vec<QuoteImport>, overwrite: bool) -> Result<Vec<QuoteImport>>;
    async fn bulk_upsert_quotes(&self, quotes: Vec<Quote>) -> Result<usize>;
    async fn save_quote(&self, quote: &Quote) -> Result<Quote>;
    
    // Repository methods exposed through service
    fn get_latest_quotes_pair_for_symbols(
        &self,
        symbol_source_pairs: &[(String, String)],
    ) -> Result<HashMap<String, LatestQuotePair>>;
}

#[async_trait]
pub trait MarketDataRepositoryTrait {
    fn get_all_historical_quotes(&self) -> Result<Vec<Quote>>;
    fn get_historical_quotes_for_symbol(&self, symbol: &str, data_source: &str) -> Result<Vec<Quote>>;
    async fn save_quotes(&self, quotes: &[Quote]) -> Result<()>;
    async fn save_quote(&self, quote: &Quote) -> Result<Quote>;
    async fn delete_quote(&self, quote_id: &str) -> Result<()>;
    async fn delete_quotes_for_symbols(&self, symbols: &[String]) -> Result<()>;
    fn get_quotes_by_source(&self, symbol: &str, source: &str) -> Result<Vec<Quote>>;
    fn get_latest_quote_for_symbol(&self, symbol: &str) -> Result<Quote>;
    fn get_latest_quotes_for_symbols(
        &self,
        symbols: &[String],
    ) -> Result<HashMap<String, Quote>>;
    fn get_latest_quotes_pair_for_symbols(
        &self,
        symbol_source_pairs: &[(String, String)],
    ) -> Result<HashMap<String, LatestQuotePair>>;
    fn get_historical_quotes_for_symbols_in_range(
        &self,
        symbols: &HashSet<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<Quote>>;
    fn get_all_historical_quotes_for_symbols(
        &self,
        symbols: &HashSet<String>,
    ) -> Result<Vec<Quote>>;
    fn get_all_historical_quotes_for_symbols_by_source(
        &self,
        symbols: &HashSet<String>,
        source: &str,
    ) -> Result<Vec<Quote>>;
    fn get_latest_sync_dates_by_source(&self) -> Result<HashMap<String, Option<NaiveDateTime>>>;
    fn get_all_providers(&self) -> Result<Vec<MarketDataProviderSetting>>;
    fn get_provider_by_id(&self, provider_id: &str) -> Result<MarketDataProviderSetting>;
    async fn update_provider_settings(
        &self,
        provider_id: String,
        changes: UpdateMarketDataProviderSetting,
    ) -> Result<MarketDataProviderSetting>;

    // --- Quote Import Methods ---
    async fn bulk_insert_quotes(&self, quote_records: Vec<QuoteDb>) -> Result<usize>;
    async fn bulk_update_quotes(&self, quote_records: Vec<QuoteDb>) -> Result<usize>;
    async fn bulk_upsert_quotes(&self, quote_records: Vec<Quote>) -> Result<usize>;
    fn quote_exists(&self, symbol_param: &str, date: &str) -> Result<bool>;
    fn get_existing_quotes_for_period(&self, symbol_param: &str, start_date: &str, end_date: &str) -> Result<Vec<Quote>>;
}
