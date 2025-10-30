use crate::market_data::market_data_constants::{
    DATA_SOURCE_MANUAL, DATA_SOURCE_MARKET_DATA_APP, DATA_SOURCE_YAHOO,
    DATA_SOURCE_ALPHA_VANTAGE, DATA_SOURCE_METAL_PRICE_API, DATA_SOURCE_VN_MARKET
};
use crate::market_data::market_data_errors::MarketDataError;
use crate::market_data::market_data_model::{
    MarketDataProviderSetting, Quote as ModelQuote, QuoteSummary,
};
use crate::market_data::providers::manual_provider::ManualProvider;
use crate::market_data::providers::market_data_provider::{AssetProfiler, MarketDataProvider};
use crate::market_data::providers::marketdata_app_provider::MarketDataAppProvider;
use crate::market_data::providers::metal_price_api_provider::MetalPriceApiProvider;
use crate::market_data::providers::alpha_vantage_provider::AlphaVantageProvider;
use crate::market_data::providers::vn_market_provider::{VnMarketProvider};
use crate::market_data::providers::yahoo_provider::YahooProvider;
use crate::secrets::SecretManager;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

pub struct ProviderRegistry {
    data_providers: HashMap<String, Arc<dyn MarketDataProvider + Send + Sync>>,
    ordered_data_provider_ids: Vec<String>,
    asset_profilers: HashMap<String, Arc<dyn AssetProfiler + Send + Sync>>,
    ordered_profiler_ids: Vec<String>,
}

impl ProviderRegistry {
    pub async fn new(
        provider_settings: Vec<MarketDataProviderSetting>,
    settings_service: Option<Arc<dyn crate::settings::SettingsServiceTrait>>,

    ) -> Result<Self, MarketDataError> {
        let mut active_providers_with_priority: Vec<(
            i32,
            String,
            Arc<dyn MarketDataProvider + Send + Sync>,
            Option<Arc<dyn AssetProfiler + Send + Sync>>,
        )> = Vec::new();

        for setting in provider_settings {
            if !setting.enabled {
                info!(
                    "Provider '{}' (ID: {}) is disabled, skipping.",
                    setting.name, setting.id
                );
                continue;
            }

            let provider_id_str = &setting.id;

            let api_key = if provider_id_str != DATA_SOURCE_YAHOO 
                && provider_id_str != DATA_SOURCE_VN_MARKET {
                match SecretManager::get_secret(provider_id_str) {
                    Ok(key_opt) => key_opt,
                    Err(e) => {
                        warn!(
                            "Failed to resolve API key for provider '{}' (ID: {}). Error: {}. Skipping.",
                            setting.name, setting.id, e
                        );
                        continue;
                    }
                }
            } else {
                None
            };

            let (provider, profiler) = match provider_id_str.as_str() {
                DATA_SOURCE_YAHOO => {
                    let p = Arc::new(YahooProvider::new().await?);
                    (
                        Some(p.clone() as Arc<dyn MarketDataProvider + Send + Sync>),
                        Some(p as Arc<dyn AssetProfiler + Send + Sync>),
                    )
                }
                DATA_SOURCE_MARKET_DATA_APP => {
                    if let Some(key) = api_key {
                        if !key.is_empty() {
                            let p = Arc::new(MarketDataAppProvider::new(key).await?);
                            (
                                Some(p.clone() as Arc<dyn MarketDataProvider + Send + Sync>),
                                None,
                            )
                        } else {
                            warn!("MarketData.app provider '{}' (ID: {}) is enabled but API key is empty. Skipping.", setting.name, setting.id);
                            (None, None)
                        }
                    } else {
                        warn!("MarketData.app provider '{}' (ID: {}) is enabled but requires an API key, which was not found or resolved. Skipping.", setting.name, setting.id);
                        (None, None)
                    }
                }
                DATA_SOURCE_ALPHA_VANTAGE => {
                    if let Some(key) = api_key {
                        if !key.is_empty() {
                            let p = Arc::new(AlphaVantageProvider::new(key));
                            (
                                Some(p.clone() as Arc<dyn MarketDataProvider + Send + Sync>),
                                Some(p as Arc<dyn AssetProfiler + Send + Sync>),
                            )
                        } else {
                            warn!("AlphaVantage provider '{}' (ID: {}) is enabled but API key is empty. Skipping.", setting.name, setting.id);
                            (None, None)
                        }
                    } else {
                        warn!("AlphaVantage provider '{}' (ID: {}) is enabled but requires an API key, which was not found or resolved. Skipping.", setting.name, setting.id);
                        (None, None)
                    }
                }
                DATA_SOURCE_METAL_PRICE_API => {
                    if let Some(key) = api_key {
                        if !key.is_empty() {
                            let p = Arc::new(MetalPriceApiProvider::new(key));
                            (
                                Some(p.clone() as Arc<dyn MarketDataProvider + Send + Sync>),
                                Some(p as Arc<dyn AssetProfiler + Send + Sync>),
                            )
                        } else {
                            warn!("MetalPriceApi provider '{}' (ID: {}) is enabled but API key is empty. Skipping.", setting.name, setting.id);
                            (None, None)
                        }
                    } else {
                        warn!("MetalPriceApi provider '{}' (ID: {}) is enabled but requires an API key, which was not found or resolved. Skipping.", setting.name, setting.id);
                        (None, None)
                    }
                }
                DATA_SOURCE_VN_MARKET => {
                    let vn_url = if let Some(ref settings) = settings_service {
                        settings.get_vn_market_service_url().unwrap_or_else(|_| "http://127.0.0.1:8765".to_string())
                    } else {
                        "http://127.0.0.1:8765".to_string()
                    };
                    let p = Arc::new(VnMarketProvider::new().with_base_url(vn_url));
                    (
                        Some(p.clone() as Arc<dyn MarketDataProvider + Send + Sync>),
                        Some(p as Arc<dyn AssetProfiler + Send + Sync>),
                    )
                }
                DATA_SOURCE_MANUAL => {
                    let p = Arc::new(ManualProvider::new()?);
                    (
                        Some(p.clone() as Arc<dyn MarketDataProvider + Send + Sync>),
                        Some(p as Arc<dyn AssetProfiler + Send + Sync>),
                    )
                }
                _ => {
                    warn!("Unknown provider ID: {}", provider_id_str);
                    (None, None)
                }
            };

            if let (Some(provider), profiler) = (provider, profiler) {
                active_providers_with_priority.push((
                    setting.priority,
                    setting.id.clone(),
                    provider,
                    profiler,
                ));
                info!(
                    "Registered provider '{}' (ID: {}) with priority {}",
                    setting.name, setting.id, setting.priority
                );
            }
        }

        // Sort by priority (lower number = higher priority)
        active_providers_with_priority.sort_by_key(|(priority, _, _, _)| *priority);

        let mut data_providers = HashMap::new();
        let mut ordered_data_provider_ids = Vec::new();
        let mut asset_profilers = HashMap::new();
        let mut ordered_profiler_ids = Vec::new();

        for (priority, id, provider, profiler) in active_providers_with_priority {
            ordered_data_provider_ids.push(id.clone());
            data_providers.insert(id, provider);

            if let Some(profiler) = profiler {
                ordered_profiler_ids.push(ordered_data_provider_ids.last().unwrap().clone());
                asset_profilers.insert(ordered_data_provider_ids.last().unwrap().clone(), profiler);
            }
        }

        Ok(Self {
            data_providers,
            ordered_data_provider_ids,
            asset_profilers,
            ordered_profiler_ids,
        })
    }

    pub async fn get_provider_for_symbol(&self, symbol: &str) -> Option<&str> {
        // Try each provider in order of priority
        for provider_id in &self.ordered_data_provider_ids {
            if let Some(provider) = self.data_providers.get(provider_id) {
                // For now, we'll use a simple heuristic
                // In a real implementation, you might want to check if the provider
                // actually supports the symbol
                if symbol.ends_with(".VN") || symbol.ends_with(".vn") {
                    if provider_id == DATA_SOURCE_VN_MARKET {
                        return Some(provider_id);
                    }
                } else if provider_id != DATA_SOURCE_VN_MARKET {
                    return Some(provider_id);
                }
            }
        }
        None
    }

    pub async fn get_provider(&self, provider_id: &str) -> Option<Arc<dyn MarketDataProvider + Send + Sync>> {
        self.data_providers.get(provider_id).cloned()
    }

    pub async fn get_profiler(&self, provider_id: &str) -> Option<Arc<dyn AssetProfiler + Send + Sync>> {
        self.asset_profilers.get(provider_id).cloned()
    }

    pub async fn get_all_providers(&self) -> Vec<Arc<dyn MarketDataProvider + Send + Sync>> {
        self.ordered_data_provider_ids
            .iter()
            .filter_map(|id| self.data_providers.get(id).cloned())
            .collect()
    }

    fn contains_vn_indicator(query: &str) -> bool {
        query.to_uppercase().contains("VN")
    }

    pub async fn search_ticker(&self, query: &str) -> Result<Vec<QuoteSummary>, MarketDataError> {
        let mut all_results = Vec::new();
        let mut errors = Vec::new();

        // Determine profiler search order based on VN indicator in query
        let search_order = if Self::contains_vn_indicator(query) {
            // If query contains "VN", prioritize VN_MARKET
            let mut reordered = Vec::new();
            if self.ordered_profiler_ids.contains(&DATA_SOURCE_VN_MARKET.to_string()) {
                reordered.push(DATA_SOURCE_VN_MARKET.to_string());
            }
            for profiler_id in &self.ordered_profiler_ids {
                if profiler_id != DATA_SOURCE_VN_MARKET {
                    reordered.push(profiler_id.clone());
                }
            }
            reordered
        } else {
            // Use default priority order
            self.ordered_profiler_ids.clone()
        };

        // Try each profiler in determined order
        for profiler_id in &search_order {
            if let Some(profiler) = self.asset_profilers.get(profiler_id) {
                match profiler.search_ticker(query).await {
                    Ok(mut results) => {
                        debug!(
                            "Found {} results from profiler '{}'",
                            results.len(),
                            profiler_id
                        );
                        all_results.append(&mut results);
                        
                        // Only break if we got results OR if this is a VN-prioritized search
                        // and we're at VN_MARKET (even with 0 results, respect VN priority)
                        if !results.is_empty() || (Self::contains_vn_indicator(query) && profiler_id == DATA_SOURCE_VN_MARKET) {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to search ticker '{}' with profiler '{}': {}",
                            query, profiler_id, e
                        );
                        errors.push((profiler_id.clone(), e));
                    }
                }
            }
        }

        if all_results.is_empty() && !errors.is_empty() {
            return Err(MarketDataError::ProviderError(format!(
                "All profilers failed. Last error: {:?}",
                errors.last().map(|(_, e)| e)
            )));
        }

        // Remove duplicates and sort by score
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all_results.dedup_by(|a, b| a.symbol == b.symbol);
        all_results.truncate(10); // Limit to top 10 results

        Ok(all_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_vn_indicator() {
        // Queries with VN indicator
        assert!(ProviderRegistry::contains_vn_indicator("VN"));
        assert!(ProviderRegistry::contains_vn_indicator("vn"));
        assert!(ProviderRegistry::contains_vn_indicator("MBB.VN"));
        assert!(ProviderRegistry::contains_vn_indicator("FPT.vn"));
        assert!(ProviderRegistry::contains_vn_indicator("VN30"));
        assert!(ProviderRegistry::contains_vn_indicator("VNINDEX"));
        assert!(ProviderRegistry::contains_vn_indicator("vn_gold"));
        assert!(ProviderRegistry::contains_vn_indicator("VN_OIL"));
        assert!(ProviderRegistry::contains_vn_indicator("HNX"));
        assert!(ProviderRegistry::contains_vn_indicator("UPCOM"));

        // Queries without VN indicator
        assert!(!ProviderRegistry::contains_vn_indicator("FPT"));
        assert!(!ProviderRegistry::contains_vn_indicator("GOLD"));
        assert!(!ProviderRegistry::contains_vn_indicator("SILVER"));
        assert!(!ProviderRegistry::contains_vn_indicator("AAPL"));
        assert!(!ProviderRegistry::contains_vn_indicator("MSFT"));
        assert!(!ProviderRegistry::contains_vn_indicator(""));
        assert!(!ProviderRegistry::contains_vn_indicator("XAU"));
    }
}
