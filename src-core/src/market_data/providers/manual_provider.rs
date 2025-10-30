use crate::market_data::market_data_model::DataSource;
use crate::market_data::providers::market_data_provider::{AssetProfiler, MarketDataProvider};
use crate::market_data::market_data_errors::MarketDataError;
use crate::market_data::QuoteSummary;
use crate::market_data::market_data_model::Quote;
use std::time::SystemTime;

use super::models::AssetProfile;
pub struct ManualProvider;

impl ManualProvider {
    pub fn new() -> Result<Self, MarketDataError> {
        Ok(ManualProvider)
    }
}

#[async_trait::async_trait]
impl MarketDataProvider for ManualProvider {
    fn name(&self) -> &'static str {
        "Manual"
    }

    async fn get_latest_quote(&self, _symbol: &str, _fallback_currency: String) -> Result<Quote, MarketDataError> {
        Err(MarketDataError::UnsupportedProvider("Manual provider does not support quote fetching".to_string()))
    }

    async fn get_historical_quotes(
        &self,
        _symbol: &str,
        _start: SystemTime,
        _end: SystemTime,
        _fallback_currency: String,
    ) -> Result<Vec<Quote>, MarketDataError> {
        Err(MarketDataError::UnsupportedProvider("Manual provider does not support historical quotes".to_string()))
    }

    async fn get_historical_quotes_bulk(
        &self,
        _symbols_with_currencies: &[(String, String, Option<String>)],
        _start: SystemTime,
        _end: SystemTime,
    ) -> Result<(Vec<Quote>, Vec<(String, String, Option<String>)>), MarketDataError> {
        Err(MarketDataError::UnsupportedProvider("Manual provider does not support bulk historical quotes".to_string()))
    }
}

#[async_trait::async_trait]
impl AssetProfiler for ManualProvider {
    async fn get_asset_profile(&self, symbol: &str) -> Result<AssetProfile, MarketDataError> {
        if symbol.starts_with("$CASH-") {
            Ok(AssetProfile {
                id: Some(symbol.to_string()),
                isin: None,
                name: Some(symbol.to_string()),
                asset_type: Some("CASH".to_string()),
                asset_class: Some("CASH".to_string()),
                asset_sub_class: Some("CASH".to_string()),
                symbol: symbol.to_string(),
                data_source: DataSource::Manual.as_str().to_string(),
                currency: symbol[6..].to_string(),
                ..Default::default()
            })
        } else {
            Ok(AssetProfile {
                id: Some(symbol.to_string()),
                isin: None,
                name: Some(symbol.to_string()),
                asset_type: Some("EQUITY".to_string()),
                asset_class: Some("EQUITY".to_string()),
                asset_sub_class: Some("COMMON_STOCK".to_string()),
                symbol: symbol.to_string(),
                data_source: DataSource::Manual.as_str().to_string(),
                ..Default::default()
            })
        }
    }

    async fn search_ticker(&self, _query: &str) -> Result<Vec<QuoteSummary>, MarketDataError> {
        Ok(vec![])
    }
}