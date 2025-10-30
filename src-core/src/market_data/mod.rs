pub(crate) mod market_data_constants;
pub(crate) mod market_data_errors;
pub mod market_data_model;
pub(crate) mod market_data_repository;
pub(crate) mod market_data_service;
pub mod market_data_traits;
pub mod providers;

// Re-export the public interface
pub use market_data_constants::*;
pub use market_data_model::{Quote, QuoteSummary, QuoteRequest, DataSource, MarketDataProviderInfo, MarketDataProviderSetting, QuoteImport, ImportValidationStatus};
pub use market_data_repository::MarketDataRepository;
pub use market_data_service::MarketDataService;
pub use market_data_traits::{MarketDataServiceTrait, MarketDataRepositoryTrait};

// Re-export provider types
pub use providers::market_data_provider::{MarketDataProvider, AssetProfiler};
pub use providers::provider_registry::ProviderRegistry;

// Re-export error types for convenience
pub use market_data_errors::MarketDataError;
