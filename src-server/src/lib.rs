//! WealthVN web server library.
//!
//! Provides HTTP API endpoints for the WealthVN portfolio tracker
//! using Axum for the web framework.

mod ai_environment;
mod api;
mod config;

pub use config::Config;

use ai_environment::ServerAiEnvironment;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};
use wealthvn_ai::{ChatConfig, ChatService};
use wealthvn_core::{
    accounts::{AccountRepository, AccountService},
    activities::{ActivityRepository, ActivityService},
    db::{self, write_actor},
    fx::{FxRepository, FxService, FxServiceTrait},
    goals::{GoalRepository, GoalService},
    market_data::{MarketDataRepository, MarketDataService},
    portfolio::{
        holdings::{HoldingsService, HoldingsValuationService},
        income::IncomeService,
        performance::PerformanceService,
    },
    settings::{settings_repository::SettingsRepository, SettingsService},
    snapshot::{SnapshotRepository, SnapshotService},
    valuation::{ValuationRepository, ValuationService},
    AssetRepository, AssetService,
};

use wealthvn_ai::{AiProviderService, AiProviderServiceTrait};

/// Application state shared across all request handlers.
pub struct AppState {
    /// Base currency for portfolio valuation
    pub base_currency: Arc<RwLock<String>>,

    /// Account service
    pub account_service: Arc<dyn wealthvn_core::accounts::AccountServiceTrait + Send + Sync>,

    /// Activity service
    pub activity_service: Arc<dyn wealthvn_core::activities::ActivityServiceTrait + Send + Sync>,

    /// Asset service
    pub asset_service: Arc<wealthvn_core::AssetService>,

    /// Goal service
    pub goal_service: Arc<dyn wealthvn_core::goals::GoalServiceTrait + Send + Sync>,

    /// Market data service
    pub market_data_service: Arc<dyn wealthvn_core::market_data::MarketDataServiceTrait + Send + Sync>,

    /// Holdings service
    pub holdings_service: Arc<dyn wealthvn_core::portfolio::holdings::HoldingsServiceTrait + Send + Sync>,

    /// Valuation service
    pub valuation_service: Arc<dyn wealthvn_core::portfolio::valuation::ValuationServiceTrait + Send + Sync>,

    /// Performance service
    pub performance_service: Arc<dyn wealthvn_core::portfolio::performance::PerformanceServiceTrait + Send + Sync>,

    /// Income service
    pub income_service: Arc<dyn wealthvn_core::portfolio::income::IncomeServiceTrait + Send + Sync>,

    /// Snapshot service
    pub snapshot_service: Arc<dyn wealthvn_core::portfolio::snapshot::SnapshotServiceTrait + Send + Sync>,

    /// Settings service
    pub settings_service: Arc<dyn wealthvn_core::settings::SettingsServiceTrait + Send + Sync>,

    /// FX service
    pub fx_service: Arc<dyn FxServiceTrait + Send + Sync>,

    /// AI provider service
    pub ai_provider_service: Arc<dyn AiProviderServiceTrait + Send + Sync>,

    /// AI chat service
    pub ai_chat_service: Arc<ChatService<ServerAiEnvironment>>,

    /// Data root directory
    pub data_root: String,

    /// Database path
    pub db_path: String,

    /// Instance ID
    pub instance_id: String,
}

/// Initialize tracing for logging.
pub fn init_tracing() {
    let log_format = std::env::var("WF_LOG_FORMAT").unwrap_or_else(|_| "text".to_string());
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let registry = tracing_subscriber::registry().with(filter);

    if log_format.eq_ignore_ascii_case("json") {
        registry
            .with(fmt::layer().json().with_current_span(false))
            .init();
    } else {
        registry
            .with(fmt::layer().with_target(true).with_line_number(true))
            .init();
    }
}

/// Build the application state with all services initialized.
pub async fn build_state(config: &Config) -> anyhow::Result<Arc<AppState>> {
    // Ensure DATABASE_URL aligns with WF_DB_PATH so core picks the right file
    std::env::set_var("DATABASE_URL", &config.db_path);
    let db_path = wealthvn_core::db::init(&config.db_path)?;
    info!("Database path in use: {}", db_path);

    let pool = db::create_pool(&db_path)?;
    let writer = write_actor::spawn_writer(pool.as_ref().clone());

    // Run database migrations
    db::run_migrations(&pool)?;
    info!("Database migrations completed");

    // Get data root path
    let data_root_path = std::path::Path::new(&db_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    // Resolve secret path
    let resolved_secret_path = std::env::var("WF_SECRET_FILE")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| data_root_path.join("secrets.json"));

    // Instantiate Repositories
    let settings_repository = Arc::new(SettingsRepository::new(pool.clone(), writer.clone()));
    let account_repository = Arc::new(AccountRepository::new(pool.clone(), writer.clone()));
    let activity_repository = Arc::new(ActivityRepository::new(pool.clone(), writer.clone()));
    let asset_repository = Arc::new(AssetRepository::new(pool.clone(), writer.clone()));
    let goal_repo = Arc::new(GoalRepository::new(pool.clone(), writer.clone()));
    let market_data_repo = Arc::new(MarketDataRepository::new(pool.clone(), writer.clone()));
    let fx_repository = Arc::new(FxRepository::new(pool.clone(), writer.clone()));
    let snapshot_repository = Arc::new(SnapshotRepository::new(pool.clone(), writer.clone()));
    let valuation_repository = Arc::new(ValuationRepository::new(pool.clone(), writer.clone()));

    // Transaction executor
    let transaction_executor = pool.clone();

    // Build services
    let fx_service = Arc::new(FxService::new(fx_repository.clone()));
    fx_service.initialize()?;

    let settings_service: Arc<dyn wealthvn_core::settings::SettingsServiceTrait + Send + Sync> =
        Arc::new(SettingsService::new(
            settings_repository.clone(),
            fx_service.clone(),
        ));

    let settings = settings_service.get_settings()?;
    let base_currency_string = settings.base_currency.clone();
    let base_currency = Arc::new(RwLock::new(base_currency_string.clone()));
    let instance_id = settings.instance_id.clone();

    let market_data_service: Arc<dyn wealthvn_core::market_data::MarketDataServiceTrait + Send + Sync> =
        Arc::new(
            MarketDataService::with_pool(
                market_data_repo.clone(),
                asset_repository.clone(),
                Some(pool.clone()),
            )
            .await?,
        );

    let asset_service = Arc::new(AssetService::new(
        asset_repository.clone(),
        market_data_service.clone(),
        market_data_repo.clone(),
    )?);

    let account_service: Arc<dyn wealthvn_core::accounts::AccountServiceTrait + Send + Sync> =
        Arc::new(AccountService::new(
            account_repository.clone(),
            fx_service.clone(),
            transaction_executor.clone(),
            base_currency.clone(),
        ));

    let activity_service: Arc<dyn wealthvn_core::activities::ActivityServiceTrait + Send + Sync> =
        Arc::new(ActivityService::new(
            activity_repository.clone(),
            account_service.clone(),
            asset_service.clone(),
            fx_service.clone(),
            market_data_service.clone(),
        ));

    let goal_service: Arc<dyn wealthvn_core::goals::GoalServiceTrait + Send + Sync> =
        Arc::new(GoalService::new(goal_repo.clone()));

    let income_service: Arc<dyn wealthvn_core::portfolio::income::IncomeServiceTrait + Send + Sync> =
        Arc::new(IncomeService::new(
            fx_service.clone(),
            activity_repository.clone(),
            base_currency.clone(),
        ));

    let snapshot_service: Arc<dyn wealthvn_core::portfolio::snapshot::SnapshotServiceTrait + Send + Sync> =
        Arc::new(SnapshotService::new(
            base_currency.clone(),
            account_repository.clone(),
            activity_repository.clone(),
            snapshot_repository.clone(),
            asset_repository.clone(),
            fx_service.clone(),
        ));

    let holdings_valuation_service = Arc::new(HoldingsValuationService::new(
        fx_service.clone(),
        market_data_service.clone(),
    ));

    let valuation_service: Arc<dyn wealthvn_core::portfolio::valuation::ValuationServiceTrait + Send + Sync> =
        Arc::new(ValuationService::new(
            base_currency.clone(),
            valuation_repository.clone(),
            snapshot_service.clone(),
            market_data_service.clone(),
            fx_service.clone(),
        ));

    let performance_service: Arc<dyn wealthvn_core::portfolio::performance::PerformanceServiceTrait + Send + Sync> =
        Arc::new(PerformanceService::new(
            valuation_service.clone(),
            market_data_service.clone(),
        ));

    let holdings_service: Arc<dyn wealthvn_core::portfolio::holdings::HoldingsServiceTrait + Send + Sync> =
        Arc::new(HoldingsService::new(
            asset_service.clone(),
            snapshot_service.clone(),
            holdings_valuation_service.clone(),
        ));

    // Build AI provider service
    let ai_provider_service: Arc<dyn AiProviderServiceTrait + Send + Sync> = Arc::new(
        AiProviderService::new(
            settings_service.clone(),
            wealthvn_ai::FsSecretStore::new(resolved_secret_path.clone()),
        ),
    );

    // Build AI chat repository
    let ai_chat_repository = Arc::new(wealthvn_ai::SqliteChatRepository::new(&db_path)?);

    // Create server AI environment
    let ai_environment = Arc::new(ServerAiEnvironment::new(
        base_currency.clone(),
        account_service.clone(),
        activity_service.clone(),
        holdings_service.clone(),
        valuation_service.clone(),
        goal_service.clone(),
        settings_service.clone(),
        Arc::new(wealthvn_ai::FsSecretStore::new(resolved_secret_path)),
        ai_chat_repository.clone(),
        performance_service.clone(),
    ));

    // Create AI chat service
    let ai_chat_service = Arc::new(ChatService::new(
        ai_environment,
        ChatConfig::default(),
    ));

    let state = AppState {
        base_currency,
        account_service,
        activity_service,
        asset_service,
        goal_service,
        market_data_service,
        holdings_service,
        valuation_service,
        performance_service,
        income_service,
        snapshot_service,
        settings_service,
        fx_service,
        ai_provider_service,
        ai_chat_service,
        data_root: data_root_path.to_string_lossy().to_string(),
        db_path,
        instance_id,
    };

    info!("Application state initialized");
    Ok(Arc::new(state))
}

pub use api::app_router;
