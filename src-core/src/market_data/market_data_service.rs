use async_trait::async_trait;
use uuid::Uuid;
use chrono::{NaiveDate, TimeZone, Utc};
use log::{debug, error};
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;


use super::market_data_model::{
    LatestQuotePair, MarketDataProviderInfo, MarketDataProviderSetting, Quote, QuoteRequest,
    QuoteSummary, UpdateMarketDataProviderSetting, QuoteImport, ImportValidationStatus, DataSource,
};
use super::market_data_traits::{MarketDataRepositoryTrait, MarketDataServiceTrait};
use super::providers::models::AssetProfile;

use crate::assets::assets_traits::AssetRepositoryTrait;
use crate::errors::Result;
use crate::market_data::providers::ProviderRegistry;

use crate::settings::SettingsServiceTrait;

const QUOTE_LOOKBACK_DAYS: i64 = 7;

pub struct MarketDataService {
    settings_service: Option<Arc<dyn SettingsServiceTrait>>,
    provider_registry: Arc<RwLock<ProviderRegistry>>,
    repository: Arc<dyn MarketDataRepositoryTrait + Send + Sync>,
    asset_repository: Arc<dyn AssetRepositoryTrait + Send + Sync>,
}

#[async_trait]
impl MarketDataServiceTrait for MarketDataService {
    async fn search_symbol(&self, query: &str) -> Result<Vec<QuoteSummary>> {
        self.provider_registry
            .read()
            .await
            .search_ticker(query)
            .await
            .map_err(|e| {
                error!("Failed to search symbol '{}': {}", query, e);
                e.into()
            })
    }

    async fn get_latest_quote(
        &self,
        symbol: &str,
        currency: &str,
    ) -> Result<Option<Quote>> {
        let quote_request = QuoteRequest {
            symbol: symbol.to_string(),
            currency: currency.to_string(),
            data_source: crate::market_data::market_data_model::DataSource::Yahoo,
        };

        self.get_latest_quotes_bulk(&[quote_request])
            .await
            .map(|quotes| quotes.into_iter().next())
    }

    async fn get_latest_quotes_bulk(
        &self,
        quote_requests: &[QuoteRequest],
    ) -> Result<Vec<Quote>> {
        let mut results = Vec::new();
        let mut failed_requests = Vec::new();

        // Group requests by provider for efficiency
        let mut provider_requests: HashMap<String, Vec<QuoteRequest>> = HashMap::new();

        let provider_registry = self.provider_registry.read().await;
        for request in quote_requests {
            let provider = provider_registry
                .get_provider_for_symbol(&request.symbol)
                .await;

            match provider {
                Some(provider_name) => {
                    provider_requests
                        .entry(provider_name.to_string())
                        .or_default()
                        .push(request.clone());
                }
                None => {
                    error!("No provider found for symbol: {}", request.symbol);
                    failed_requests.push(request.clone());
                }
            }
        }

        // Process requests for each provider
        for (provider_name, requests) in provider_requests {
            let provider = self.provider_registry.read().await.get_provider(&provider_name).await;
            
            if let Some(provider) = provider {
                let start = SystemTime::now() - std::time::Duration::from_secs((QUOTE_LOOKBACK_DAYS * 24 * 60 * 60) as u64);
                let end = SystemTime::now();

                let symbols_with_currencies: Vec<(String, String, Option<String>)> = requests
                    .iter()
                    .map(|req| (req.symbol.clone(), req.currency.clone(), None))
                    .collect();

                match provider
                    .get_historical_quotes_bulk(&symbols_with_currencies, start, end)
                    .await
                {
                    Ok((quotes, failed_symbols)) => {
                        // Get the latest quote for each symbol
                        let mut latest_quotes: HashMap<String, Quote> = HashMap::new();
                        
                        for quote in quotes {
                            let entry = latest_quotes.entry(quote.symbol.clone());
                            entry
                                .and_modify(|existing| {
                                    if quote.timestamp > existing.timestamp {
                                        *existing = quote.clone();
                                    }
                                })
                                .or_insert(quote);
                        }

                        results.extend(latest_quotes.into_values());
                        failed_requests.extend(
                            failed_symbols
                                .into_iter()
                                .zip(requests.iter())
                                .map(|((symbol, currency, _), req)| QuoteRequest {
                                    symbol,
                                    currency,
                                    data_source: req.data_source.clone(),
                                })
                        );
                    }
                    Err(e) => {
                        error!(
                            "Failed to get quotes from provider '{}': {}",
                            provider_name, e
                        );
                        failed_requests.extend(requests.clone());
                    }
                }
            }
        }

        if failed_requests.is_empty() {
            Ok(results)
        } else {
            Err(crate::errors::Error::MarketData(
                crate::market_data::MarketDataError::ProviderError(format!(
                    "Failed to fetch {} quotes",
                    failed_requests.len()
                )),
            ))
        }
    }

    async fn get_historical_quotes(
        &self,
        symbol: &str,
        start: SystemTime,
        end: SystemTime,
        currency: &str,
    ) -> Result<Vec<Quote>> {
        let provider_registry = self.provider_registry.read().await;
        let provider = provider_registry
            .get_provider_for_symbol(symbol)
            .await;

        match provider {
            Some(provider_name) => {
                let provider = provider_registry
                    .get_provider(provider_name)
                    .await;

                if let Some(provider) = provider {
                    provider
                        .get_historical_quotes(symbol, start, end, currency.to_string())
                        .await
                        .map_err(|e| {
                            error!(
                                "Failed to get historical quotes for '{}' from provider '{}': {}",
                                symbol, provider_name, e
                            );
                            e.into()
                        })
                } else {
                    Err(crate::errors::Error::MarketData(
                        crate::market_data::MarketDataError::NotFound(symbol.to_string()),
                    ))
                }
            }
            None => Err(crate::errors::Error::MarketData(
                crate::market_data::MarketDataError::NotFound(symbol.to_string()),
            )),
        }
    }

    async fn get_asset_profile(&self, symbol: &str) -> Result<Option<AssetProfile>> {
        let provider_registry = self.provider_registry.read().await;
        let provider_name = provider_registry
            .get_provider_for_symbol(symbol)
            .await;

        match provider_name {
            Some(provider_name) => {
                let profiler = self
                    .provider_registry
                    .read()
                    .await
                    .get_profiler(provider_name)
                    .await;

                if let Some(profiler) = profiler {
                    match profiler.get_asset_profile(symbol).await {
                        Ok(profile) => Ok(Some(profile)),
                        Err(crate::market_data::MarketDataError::NotFound(_)) => Ok(None),
                        Err(e) => {
                            error!(
                                "Failed to get asset profile for '{}' from provider '{}': {}",
                                symbol, provider_name, e
                            );
                            Err(e.into())
                        }
                    }
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn import_quotes(&self, quotes: Vec<QuoteImport>) -> Result<Vec<ImportValidationStatus>> {
        let mut validation_results = Vec::new();

        for quote_import in &quotes {
            let validation_status = self.validate_quote_import(&quote_import).await;
            validation_results.push(validation_status);
        }

        let valid_quotes: Vec<Quote> = validation_results
            .iter()
            .zip(quotes.iter())
            .filter_map(|(status, import)| match status {
                ImportValidationStatus::Valid => {
                    // Parse the date string to DateTime<Utc>
                    let parsed_date = NaiveDate::parse_from_str(&import.date, "%Y-%m-%d")
                        .ok()?
                        .and_hms_opt(0, 0, 0)?;
                    let timestamp = Utc.from_utc_datetime(&parsed_date);
                    
                    let quote = Quote {
                        id: format!("{}_{}", import.symbol, import.date.replace("-", "")),
                        symbol: import.symbol.clone(),
                        timestamp,
                        open: import.open.unwrap_or(import.close),
                        high: import.high.unwrap_or(import.close),
                        low: import.low.unwrap_or(import.close),
                        close: import.close,
                        adjclose: import.close,
                        volume: import.volume.unwrap_or(Decimal::ZERO),
                        currency: import.currency.clone(),
                        data_source: crate::market_data::market_data_model::DataSource::Manual,
                        created_at: Utc::now(),
                    };
                    Some(quote)
                }
                _ => None,
            })
            .collect();

        if !valid_quotes.is_empty() {
            self.repository.save_quotes(&valid_quotes).await?;
        }

        Ok(validation_results)
    }

    async fn import_quotes_from_csv(
        &self,
        mut quotes: Vec<QuoteImport>,
        overwrite: bool,
    ) -> Result<Vec<QuoteImport>> {
        for quote in quotes.iter_mut() {
            // Parse date string to NaiveDateTime
            let parsed_date = match NaiveDate::parse_from_str(&quote.date, "%Y-%m-%d") {
                Ok(date) => date.and_hms_opt(0, 0, 0),
                Err(_) => {
                    quote.validation_status = ImportValidationStatus::Error(
                        format!("Invalid date format: {}", quote.date)
                    );
                    quote.error_message = Some(format!("Invalid date format: {}", quote.date));
                    continue;
                }
            };

            let naive_datetime = match parsed_date {
                Some(dt) => dt,
                None => {
                    quote.validation_status = ImportValidationStatus::Error(
                        "Failed to create datetime".to_string()
                    );
                    quote.error_message = Some("Failed to create datetime".to_string());
                    continue;
                }
            };

            // Check if quote exists if not overwriting
            if !overwrite {
                let date_str = &quote.date;
                match self.repository.quote_exists(&quote.symbol, date_str) {
                    Ok(true) => {
                        quote.validation_status = ImportValidationStatus::Warning(
                            "Quote already exists".to_string()
                        );
                        quote.error_message = Some("Quote already exists (use overwrite to replace)".to_string());
                        continue;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        quote.validation_status = ImportValidationStatus::Error(
                            format!("Database error: {}", e)
                        );
                        quote.error_message = Some(format!("Database error: {}", e));
                        continue;
                    }
                }
            }

            // Create Quote from QuoteImport
            let new_quote = Quote {
                id: Uuid::new_v4().to_string(),
                created_at: Utc::now(),
                data_source: DataSource::Manual,
                timestamp: Utc.from_utc_datetime(&naive_datetime),
                symbol: quote.symbol.clone(),
                open: quote.open.unwrap_or(Decimal::ZERO),
                high: quote.high.unwrap_or(Decimal::ZERO),
                low: quote.low.unwrap_or(Decimal::ZERO),
                volume: quote.volume.unwrap_or(Decimal::ZERO),
                close: quote.close,
                adjclose: quote.close,
                currency: quote.currency.clone(),
            };

            // Save the quote
            match self.repository.save_quote(&new_quote).await {
                Ok(_) => {
                    quote.validation_status = ImportValidationStatus::Valid;
                }
                Err(e) => {
                    quote.validation_status = ImportValidationStatus::Error(
                        format!("Failed to save: {}", e)
                    );
                    quote.error_message = Some(format!("Failed to save: {}", e));
                }
            }
        }

        Ok(quotes)
    }

    async fn bulk_upsert_quotes(&self, quotes: Vec<Quote>) -> Result<usize> {
        self.repository.bulk_upsert_quotes(quotes).await
    }

    async fn validate_quote_import(&self, quote_import: &QuoteImport) -> ImportValidationStatus {
        // Validate symbol exists
        let assets = self
            .asset_repository
            .list_by_symbols(&vec![quote_import.symbol.clone()]);

        if assets.is_err() || assets.as_ref().unwrap().is_empty() {
            return ImportValidationStatus::Error(format!(
                "Asset '{}' not found",
                quote_import.symbol
            ));
        }

        // Validate price data - check optional fields
        if quote_import.open.map_or(false, |p| p.is_sign_negative())
            || quote_import.high.map_or(false, |p| p.is_sign_negative())
            || quote_import.low.map_or(false, |p| p.is_sign_negative())
            || quote_import.close.is_sign_negative()
        {
            return ImportValidationStatus::Error(
                "Prices cannot be negative".to_string(),
            );
        }

        // Validate price relationships - only if values are present
        if let (Some(high), Some(open)) = (quote_import.high, quote_import.open) {
            if high < open {
                return ImportValidationStatus::Error(
                    "High price must be >= open price".to_string(),
                );
            }
        }
        
        if let Some(high) = quote_import.high {
            if high < quote_import.close {
                return ImportValidationStatus::Error(
                    "High price must be >= close price".to_string(),
                );
            }
        }
        
        if let (Some(high), Some(low)) = (quote_import.high, quote_import.low) {
            if high < low {
                return ImportValidationStatus::Error(
                    "High price must be >= low price".to_string(),
                );
            }
        }

        if let (Some(low), Some(open)) = (quote_import.low, quote_import.open) {
            if low > open {
                return ImportValidationStatus::Error(
                    "Low price must be <= open price".to_string(),
                );
            }
        }
        
        if let Some(low) = quote_import.low {
            if low > quote_import.close {
                return ImportValidationStatus::Error(
                    "Low price must be <= close price".to_string(),
                );
            }
        }

        // Validate volume
        if quote_import.volume.map_or(false, |v| v.is_sign_negative()) {
            return ImportValidationStatus::Error(
                "Volume cannot be negative".to_string(),
            );
        }

        ImportValidationStatus::Valid
    }

    async fn get_provider_settings(&self) -> Result<Vec<MarketDataProviderSetting>> {
        self.repository.get_all_providers()
    }

    async fn update_provider_setting(
        &self,
        provider_id: String,
        update: UpdateMarketDataProviderSetting,
    ) -> Result<()> {
        debug!(
            "Updating provider setting for {}: priority: {:?}, enabled: {:?}",
            provider_id, update.priority, update.enabled
        );
        
        // Update the provider settings in the repository
        self.repository.update_provider_settings(
            provider_id,
            update
        ).await?;
        
        Ok(())
    }

    async fn get_provider_info(&self) -> Result<Vec<MarketDataProviderInfo>> {
        let providers = self.provider_registry.read().await.get_all_providers_with_ids().await;
        let mut info = Vec::new();

        for (id, provider) in providers {
            let provider_info = MarketDataProviderInfo {
                id: id.clone(),
                name: provider.name().to_string(),
                logo_filename: format!("{}.png", id.to_lowercase()),
                last_synced_date: None,
            };
            info.push(provider_info);
        }

        Ok(info)
    }
    
    async fn save_quote(&self, quote: &Quote) -> Result<Quote> {
        self.repository.save_quote(quote).await
    }
    
    fn get_latest_quotes_pair_for_symbols(
        &self,
        symbol_source_pairs: &[(String, String)],
    ) -> Result<HashMap<String, LatestQuotePair>> {
        self.repository.get_latest_quotes_pair_for_symbols(symbol_source_pairs)
    }
    
    async fn get_historical_quotes_for_symbols_in_range(
        &self,
        symbols: &HashSet<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<Quote>> {
        self.repository.get_historical_quotes_for_symbols_in_range(symbols, start_date, end_date)
    }
}

impl MarketDataService {
    pub fn new(
        settings_service: Option<Arc<dyn SettingsServiceTrait>>,
        provider_registry: Arc<RwLock<ProviderRegistry>>,
        repository: Arc<dyn MarketDataRepositoryTrait + Send + Sync>,
        asset_repository: Arc<dyn AssetRepositoryTrait + Send + Sync>,
    ) -> Self {
        Self {
            settings_service,
            provider_registry,
            repository,
            asset_repository,
        }
    }
}
