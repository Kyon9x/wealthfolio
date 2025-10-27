use async_trait::async_trait;
use reqwest::Client;
use std::time::SystemTime;
use crate::market_data::{MarketDataError, Quote as ModelQuote, AssetProfiler, QuoteSummary};
use crate::market_data::providers::market_data_provider::MarketDataProvider;
use chrono::{Utc, NaiveDate, TimeZone};
use rust_decimal::Decimal;
use serde::Deserialize;
use crate::market_data::providers::models::AssetProfile;
use crate::market_data::market_data_model::DataSource;

const BASE_URL: &str = "http://127.0.0.1:8765";

pub struct VnMarketProvider {
    client: Client,
}

impl VnMarketProvider {
    pub fn new() -> Self {
        VnMarketProvider {
            client: Client::builder()
                .no_proxy()
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Normalize symbol by stripping .VN suffix for VN Market Service API calls
    /// Example: "MBB.VN" -> "MBB", "FPT.VN" -> "FPT"
    fn normalize_symbol(symbol: &str) -> &str {
        symbol.strip_suffix(".VN").unwrap_or(symbol)
    }
}

#[derive(Debug, Deserialize)]
struct HistoryResponse {
    symbol: String,
    history: Vec<HistoryItem>,
    currency: String,
}

#[derive(Debug, Deserialize)]
struct HistoryItem {
    date: String,
    nav: f64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    symbol: String,
    fund_name: String,
    fund_type: Option<String>,
    management_company: Option<String>,
    inception_date: Option<String>,
    nav_per_unit: Option<f64>,
    currency: String,
}

#[derive(Debug, Deserialize)]
struct FundBasicInfo {
    symbol: String,
    fund_name: String,
    asset_type: String,
    data_source: String,
}

#[derive(Debug, Deserialize)]
struct FundListResponse {
    funds: Vec<FundBasicInfo>,
    total: usize,
}

#[derive(Debug, Deserialize)]
struct UnifiedSearchResponse {
    results: Vec<UnifiedSearchResult>,
    total: usize,
}

#[derive(Debug, Deserialize)]
struct UnifiedSearchResult {
    symbol: String,
    name: String,
    asset_type: String,
    exchange: String,
    currency: String,
    data_source: String,
}

#[async_trait]
impl MarketDataProvider for VnMarketProvider {
    fn name(&self) -> &'static str {
        "VN_MARKET"
    }

    fn priority(&self) -> u8 {
        5
    }

    async fn get_latest_quote(
        &self,
        symbol: &str,
        fallback_currency: String,
    ) -> Result<ModelQuote, MarketDataError> {
        let end = SystemTime::now();
        let start = end - std::time::Duration::from_secs(7 * 24 * 60 * 60);
        
        let quotes = self.get_historical_quotes(symbol, start, end, fallback_currency).await?;
        
        quotes
            .into_iter()
            .max_by_key(|q| q.timestamp)
            .ok_or_else(|| MarketDataError::NotFound(symbol.to_string()))
    }

    async fn get_historical_quotes(
        &self,
        symbol: &str,
        _start: SystemTime,
        _end: SystemTime,
        fallback_currency: String,
    ) -> Result<Vec<ModelQuote>, MarketDataError> {
        let normalized_symbol = Self::normalize_symbol(symbol);
        let url = format!("{}/history/{}", BASE_URL, normalized_symbol);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| MarketDataError::ProviderError(format!("VnMarket API error: {}", e)))?;

        if !response.status().is_success() {
            let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(MarketDataError::ProviderError(format!("VnMarket API error: {}", error_body)));
        }

        let history_response: HistoryResponse = response
            .json()
            .await
            .map_err(|e| MarketDataError::ProviderError(format!("Failed to parse response: {}", e)))?;

        let currency = if history_response.currency.is_empty() {
            fallback_currency
        } else {
            history_response.currency
        };

        let quotes: Vec<ModelQuote> = history_response
            .history
            .into_iter()
            .filter_map(|item| {
                let date = NaiveDate::parse_from_str(&item.date, "%Y-%m-%d").ok()?;
                let timestamp = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0)?);
                
                let nav = Decimal::from_f64_retain(item.nav).unwrap_or_default();
                let open = Decimal::from_f64_retain(item.open).unwrap_or(nav);
                let high = Decimal::from_f64_retain(item.high).unwrap_or(nav);
                let low = Decimal::from_f64_retain(item.low).unwrap_or(nav);
                let close = Decimal::from_f64_retain(item.close).unwrap_or(nav);
                let volume = Decimal::from_f64_retain(item.volume).unwrap_or_default();

                let id = format!("{}_{}", timestamp.format("%Y%m%d"), symbol);

                Some(ModelQuote {
                    id,
                    symbol: symbol.to_string(),
                    timestamp,
                    open,
                    high,
                    low,
                    close,
                    adjclose: close,
                    volume,
                    currency: currency.clone(),
                    data_source: DataSource::VnMarket,
                    created_at: Utc::now(),
                })
            })
            .collect();

        if quotes.is_empty() {
            return Err(MarketDataError::NotFound(symbol.to_string()));
        }

        Ok(quotes)
    }

    async fn get_historical_quotes_bulk(
        &self,
        symbols_with_currencies: &[(String, String)],
        start: SystemTime,
        end: SystemTime,
    ) -> Result<(Vec<ModelQuote>, Vec<(String, String)>), MarketDataError> {
        let mut all_quotes = Vec::new();
        let mut failed_symbols = Vec::new();

        for (symbol, currency) in symbols_with_currencies {
            match self.get_historical_quotes(symbol, start, end, currency.clone()).await {
                Ok(mut quotes) => all_quotes.append(&mut quotes),
                Err(_) => failed_symbols.push((symbol.clone(), currency.clone())),
            }
        }

        Ok((all_quotes, failed_symbols))
    }
}

#[async_trait]
impl AssetProfiler for VnMarketProvider {
    async fn get_asset_profile(&self, symbol: &str) -> Result<AssetProfile, MarketDataError> {
        let normalized_symbol = Self::normalize_symbol(symbol);
        let url = format!("{}/search/{}", BASE_URL, normalized_symbol);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| MarketDataError::ProviderError(format!("VnMarket API error: {}", e)))?;

        if !response.status().is_success() {
            return Err(MarketDataError::NotFound(symbol.to_string()));
        }

        let search_response: SearchResponse = response
            .json()
            .await
            .map_err(|e| MarketDataError::ProviderError(format!("Failed to parse response: {}", e)))?;

        Ok(AssetProfile {
            id: None,
            isin: None,
            symbol: search_response.symbol,
            symbol_mapping: None,
            name: Some(search_response.fund_name),
            asset_type: Some("MUTUAL_FUND".to_string()),
            asset_class: Some("Equity".to_string()),
            asset_sub_class: Some("MutualFund".to_string()),
            currency: search_response.currency,
            data_source: "VN_MARKET".to_string(),
            notes: None,
            countries: None,
            categories: None,
            classes: None,
            attributes: None,
            sectors: None,
            url: None,
        })
    }

    async fn search_ticker(&self, query: &str) -> Result<Vec<QuoteSummary>, MarketDataError> {
        let url = format!("{}/search", BASE_URL);
        
        let response = self.client
            .get(&url)
            .query(&[("query", query)])
            .send()
            .await
            .map_err(|e| MarketDataError::ProviderError(format!("VnMarket API error: {}", e)))?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let search_response: UnifiedSearchResponse = response
            .json()
            .await
            .map_err(|e| MarketDataError::ProviderError(format!("Failed to parse response: {}", e)))?;

        let query_lower = query.to_lowercase();
        
        let mut results: Vec<QuoteSummary> = search_response
            .results
            .into_iter()
            .map(|result| {
                let symbol_lower = result.symbol.to_lowercase();
                let name_lower = result.name.to_lowercase();
                
                let score = if symbol_lower == query_lower {
                    1.0
                } else if symbol_lower.starts_with(&query_lower) {
                    0.9
                } else if name_lower.contains(&query_lower) {
                    0.7
                } else {
                    0.5
                };
                
                let (quote_type, type_display) = match result.asset_type.as_str() {
                    "FUND" => ("MUTUALFUND", "Mutual Fund"),
                    "STOCK" => ("EQUITY", "Stock"),
                    "INDEX" => ("INDEX", "Index"),
                    _ => ("OTHER", "Other"),
                };
                
                QuoteSummary {
                    symbol: result.symbol,
                    short_name: result.name.clone(),
                    long_name: result.name,
                    exchange: result.exchange,
                    quote_type: quote_type.to_string(),
                    type_display: type_display.to_string(),
                    index: "".to_string(),
                    score,
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        results.truncate(10);

        Ok(results)
    }
}
