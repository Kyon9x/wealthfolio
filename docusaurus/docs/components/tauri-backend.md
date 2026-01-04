---
title: Tauri Backend
sidebar_position: 2
---

# Tauri Backend

This document details the Tauri desktop backend architecture of Wealthfolio.

## Overview

Tauri provides a desktop application wrapper around the React frontend,
enabling:

- **Native OS Integration** - File system, dialogs, notifications
- **IPC (Inter-Process Communication)** - Zero-latency frontend-backend
  communication
- **Cross-platform** - Windows, macOS, Linux
- **Security** - Sandboxed execution

## Architecture

```mermaid
graph TB
    subgraph "Frontend (React)"
        UI[React UI]
        CMD[Command Wrappers]
    end

    subgraph "Tauri Runtime"
        TAURI[Tauri App]
        IPC[IPC Bridge]
    end

    subgraph "Rust Backend"
        CMDS[Command Handlers]
        CTX[Service Context]
        SERVICES[Services Layer]
        REPOS[Repositories]
        DB[SQLite Database]
        CACHE[Moka Cache]
        HTTP[HTTP Clients]
    end

    UI --> CMD
    CMD -->|invoke()| IPC
    IPC --> CMDS
    CMDS --> CTX
    CTX --> SERVICES
    SERVICES --> REPOS
    REPOS --> DB
    SERVICES --> CACHE
    SERVICES --> HTTP
```

---

## Tauri Application Structure

### Directory Layout

```
src-tauri/
├── src/
│   ├── main.rs              # Application entry
│   ├── lib.rs              # Library exports
│   ├── build.rs            # Build configuration
│   ├── commands/            # Tauri command handlers
│   │   ├── mod.rs
│   │   ├── account.rs
│   │   ├── activity.rs
│   │   ├── portfolio.rs
│   │   ├── goals.rs
│   │   ├── market_data.rs
│   │   ├── settings.rs
│   │   └── addons.rs
│   ├── context/             # Dependency injection
│   │   └── mod.rs
│   └── plugins/            # Custom Tauri plugins
├── gen/                   # Generated schemas
├── icons/                 # Application icons
├── Cargo.toml              # Rust dependencies
├── tauri.conf.json        # Tauri configuration
├── build.rs               # Build script
└── capabilities/           # Tauri capabilities
    ├── desktop.json
    └── mobile.json
```

---

## Application Initialization

### Main Entry (`src-tauri/src/main.rs`)

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::init())
        .plugin(tauri_plugin_updater::init())
        .setup(|app| {
            // 1. Get data directory
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");

            // 2. Initialize database
            let db_path = app_data_dir.join("wealthfolio.db");
            let db_pool = create_db_pool(&db_path)
                .expect("Failed to create database pool");

            // 3. Initialize service context
            let context = Arc::new(ServiceContext::new(db_pool)
                .expect("Failed to initialize service context"));

            // 4. Store context for commands
            app.manage(context.clone());

            // 5. Start background tasks
            start_background_tasks(context.clone(), app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Account commands
            commands::account::get_accounts,
            commands::account::create_account,
            commands::account::update_account,
            commands::account::delete_account,

            // Activity commands
            commands::activity::search_activities,
            commands::activity::create_activity,
            commands::activity::update_activity,
            commands::activity::delete_activity,
            commands::activity::import_activities,

            // Portfolio commands
            commands::portfolio::get_holdings,
            commands::portfolio::update_portfolio,
            commands::portfolio::calculate_performance,

            // Goal commands
            commands::goals::get_goals,
            commands::goals::create_goal,
            commands::goals::update_goal,
            commands::goals::update_goal_allocations,

            // Market data commands
            commands::market_data::sync_market_data,
            commands::market_data::get_latest_quotes,
            commands::market_data::search_symbol,

            // Settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,

            // Addon commands
            commands::addons::install_addon_zip,
            commands::addons::toggle_addon,
            commands::addons::uninstall_addon,
            commands::addons::list_installed_addons,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Library Entry (`src-tauri/src/lib.rs`)

```rust
// Re-export types for commands
pub use wealthfolio_core::types::*;
pub use wealthfolio_core::errors::*;
pub use wealthfolio_core::services::*;

// Initialize services and return ServiceContext
pub fn create_service_context(db_path: &Path) -> Result`ServiceContext>` {
    ServiceContext::new(db_path)
}
```

---

## Command Handlers

### Command Registration

All commands are registered with `#[tauri::command]` macro:

```rust
#[tauri::command]
pub async fn get_accounts(
    state: tauri::State``'_, Arc`ServiceContext>`>,
) -> Result`Vec``Account>`, String> {
    state
        .account_service
        .get_all_accounts()
        .await
        .map_err(|e| e.to_string())
}
```

### State Access

Commands access shared state via `tauri::State`:

```rust
#[tauri::command]
pub async fn create_activity(
    state: tauri::State``'_, Arc`ServiceContext>`>,
    activity: NewActivity,
) -> Result`Activity`, String> {
    state
        .activity_service
        .create(activity)
        .await
        .map_err(|e| e.to_string())
}
```

---

## Service Context

### Dependency Injection Container

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use moka::future::Cache;

pub struct ServiceContext {
    // Database
    pub db_pool: Arc`Pool>`,

    // Repositories
    pub account_repo: Arc`dyn` AccountRepository>,
    pub activity_repo: Arc`dyn` ActivityRepository>,
    pub asset_repo: Arc`dyn` AssetRepository>,
    pub quote_repo: Arc`dyn` QuoteRepository>,
    pub goal_repo: Arc`dyn` GoalRepository>,
    pub settings_repo: Arc`dyn` SettingsRepository>,
    pub addon_repo: Arc`dyn` AddonRepository>,

    // Services
    pub account_service: Arc`dyn` AccountService>,
    pub activity_service: Arc`dyn` ActivityService>,
    pub portfolio_service: Arc`dyn` PortfolioService>,
    pub market_data_service: Arc`dyn` MarketDataService>,
    pub goal_service: Arc`dyn` GoalService>,
    pub fx_service: Arc`dyn` FxService>,
    pub addon_runtime: Arc`AddonRuntime>`,

    // Caches
    pub quote_cache: Arc`Cache``String`, Quote>>,
    pub exchange_rate_cache: Arc`Cache``String`, Decimal>>,

    // HTTP Clients
    pub http_client: Arc`reqwest`::Client>,

    // Configuration
    pub config: Arc`RwLock``AppConfig>`>,
}

impl ServiceContext {
    pub fn new(db_path: &Path) -> Result`Self>` {
        // 1. Create database connection pool
        let db_pool = Arc::new(create_db_pool(db_path)?);

        // 2. Initialize repositories
        let account_repo = Arc::new(AccountRepositoryImpl::new(db_pool.clone()));
        let activity_repo = Arc::new(ActivityRepositoryImpl::new(db_pool.clone()));
        let asset_repo = Arc::new(AssetRepositoryImpl::new(db_pool.clone()));
        let quote_repo = Arc::new(QuoteRepositoryImpl::new(db_pool.clone()));
        let goal_repo = Arc::new(GoalRepositoryImpl::new(db_pool.clone()));
        let settings_repo = Arc::new(SettingsRepositoryImpl::new(db_pool.clone()));
        let addon_repo = Arc::new(AddonRepositoryImpl::new(db_pool.clone()));

        // 3. Create caches
        let quote_cache = Arc::new(
            Cache::builder()
                .time_to_live(Duration::from_secs(900)) // 15 minutes
                .max_capacity(10000)
                .build()
        );

        let exchange_rate_cache = Arc::new(
            Cache::builder()
                .time_to_live(Duration::from_secs(3600)) // 1 hour
                .max_capacity(1000)
                .build()
        );

        // 4. Initialize HTTP client
        let http_client = Arc::new(reqwest::Client::new());

        // 5. Initialize services
        let account_service = Arc::new(AccountServiceImpl::new(
            account_repo.clone()
        ));

        let activity_service = Arc::new(ActivityServiceImpl::new(
            activity_repo.clone(),
            asset_repo.clone(),
            quote_cache.clone(),
        ));

        let portfolio_service = Arc::new(PortfolioServiceImpl::new(
            activity_repo.clone(),
            asset_repo.clone(),
            quote_repo.clone(),
            exchange_rate_cache.clone(),
        ));

        let market_data_service = Arc::new(MarketDataServiceImpl::new(
            asset_repo.clone(),
            quote_repo.clone(),
            quote_cache.clone(),
            http_client.clone(),
        ));

        let goal_service = Arc::new(GoalServiceImpl::new(
            goal_repo.clone(),
            portfolio_service.clone(),
        ));

        let fx_service = Arc::new(FxServiceImpl::new(
            settings_repo.clone(),
            exchange_rate_cache.clone(),
            http_client.clone(),
        ));

        let addon_runtime = Arc::new(AddonRuntime::new(
            addon_repo.clone(),
            http_client.clone(),
        ));

        // 6. Load or create default config
        let config = Arc::new(RwLock::new(load_or_create_config(&settings_repo)?));

        Ok(Self {
            db_pool,
            account_repo,
            activity_repo,
            asset_repo,
            quote_repo,
            goal_repo,
            settings_repo,
            addon_repo,
            account_service,
            activity_service,
            portfolio_service,
            market_data_service,
            goal_service,
            fx_service,
            addon_runtime,
            quote_cache,
            exchange_rate_cache,
            http_client,
            config,
        })
    }
}
```

---

## Background Tasks

### Task Spawner

```rust
pub fn start_background_tasks(context: Arc`ServiceContext>`, app: AppHandle) {
    // 1. VN Market Sync (on startup)
    tokio::spawn(async move {
        if context.config.read().await.vn_market.auto_sync_on_startup {
            info!("Starting VN market data sync...");

            app.emit("market:sync-start", ()).ok();

            match context.market_data_service.sync_vn_market().await {
                Ok(count) => {
                    info!("VN market sync complete: {} assets", count);
                    app.emit("market:sync-complete", count).ok();
                }
                Err(e) => {
                    error!("VN market sync failed: {}", e);
                    app.emit("market:sync-error", e.to_string()).ok();
                }
            }
        }
    });

    // 2. Periodic Market Data Sync (every 15 minutes)
    let sync_context = context.clone();
    let sync_app = app.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(900));

        loop {
            interval.tick().await;

            info!("Starting periodic market data sync...");

            match sync_context.market_data_service.sync_all_assets().await {
                Ok(_) => {
                    info!("Periodic sync complete");
                }
                Err(e) => {
                    error!("Periodic sync failed: {}", e);
                }
            }
        }
    });

    // 3. Portfolio Update (periodic)
    let portfolio_context = context.clone();
    let portfolio_app = app.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour

        loop {
            interval.tick().await;

            info!("Starting portfolio update...");

            portfolio_app.emit("portfolio:update-start", ()).ok();

            match portfolio_context.portfolio_service.update_portfolio().await {
                Ok(_) => {
                    info!("Portfolio update complete");
                    portfolio_app.emit("portfolio:update-complete", ()).ok();
                }
                Err(e) => {
                    error!("Portfolio update failed: {}", e);
                    portfolio_app.emit("portfolio:update-error", e.to_string()).ok();
                }
            }
        }
    });
}
```

---

## Event System

### Event Emission

```rust
// Emit event to frontend
app.emit_all("portfolio:update-complete", ())?;

// Emit with data
app.emit_all("market:sync-complete", json!({
    "count": 150,
    "providers": ["VCI", "FMarket"]
}))?;
```

### Event Types

```rust
pub enum WealthfolioEvent {
    // Portfolio events
    PortfolioUpdateStart,
    PortfolioUpdateComplete,
    PortfolioUpdateError(String),

    // Market data events
    MarketSyncStart,
    MarketSyncComplete { count: usize },
    MarketSyncError(String),

    // Activity events
    ActivitiesImported { count: usize },
    ActivityCreated { id: i32 },
    ActivityDeleted { id: i32 },

    // Goal events
    GoalsUpdated,
    GoalCreated { id: i32 },
    GoalAllocationUpdated { id: i32 },

    // Addon events
    AddonInstalled { name: String },
    AddonEnabled { name: String },
    AddonDisabled { name: String },
    AddonUninstalled { name: String },
}

impl WealthfolioEvent {
    pub fn event_name(&self) -> &str {
        match self {
            Self::PortfolioUpdateStart => "portfolio:update-start",
            Self::PortfolioUpdateComplete => "portfolio:update-complete",
            Self::PortfolioUpdateError(_) => "portfolio:update-error",
            Self::MarketSyncStart => "market:sync-start",
            Self::MarketSyncComplete { .. } => "market:sync-complete",
            Self::MarketSyncError(_) => "market:sync-error",
            // ... other events
        }
    }
}
```

---

## Key Commands

### Account Commands

```rust
#[tauri::command]
pub async fn get_accounts(
    state: tauri::State``'_, Arc`ServiceContext>`>,
) -> Result`Vec``Account>`, String> {
    state.account_service.get_all_accounts()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_account(
    state: tauri::State``'_, Arc`ServiceContext>`>,
    account: NewAccount,
) -> Result`Account`, String> {
    state.account_service.create(account)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_account(
    state: tauri::State``'_, Arc`ServiceContext>`>,
    id: i32,
    account: UpdateAccount,
) -> Result`Account`, String> {
    state.account_service.update(id, account)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_account(
    state: tauri::State``'_, Arc`ServiceContext>`>,
    id: i32,
) -> Result``(), String> {
    state.account_service.delete(id)
        .await
        .map_err(|e| e.to_string())
}
```

### Portfolio Commands

```rust
#[tauri::command]
pub async fn get_holdings(
    state: tauri::State``'_, Arc`ServiceContext>`>,
    account_id: Option`i32>`,
) -> Result`Vec``Holding>`, String> {
    state.portfolio_service.get_holdings(account_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_portfolio(
    state: tauri::State``'_, Arc`ServiceContext>`>,
    app: AppHandle,
) -> Result``(), String> {
    app.emit_all("portfolio:update-start", ()).ok();

    state.portfolio_service.update_portfolio()
        .await
        .map_err(|e| e.to_string())?;

    app.emit_all("portfolio:update-complete", ()).ok();

    Ok(())
}

#[tauri::command]
pub async fn calculate_performance(
    state: tauri::State``'_, Arc`ServiceContext>`>,
    account_id: Option`i32>`,
    start_date: Option`String>`,
    end_date: Option`String>`,
) -> Result`PerformanceSummary`, String> {
    let start = start_date
        .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
        .transpose()?;

    let end = end_date
        .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
        .transpose()?;

    state.portfolio_service.calculate_performance(account_id, start, end)
        .await
        .map_err(|e| e.to_string())
}
```

### Market Data Commands

```rust
#[tauri::command]
pub async fn sync_market_data(
    state: tauri::State``'_, Arc`ServiceContext>`>,
    app: AppHandle,
) -> Result`SyncResult`, String> {
    app.emit_all("market:sync-start", ()).ok();

    let result = state.market_data_service.sync_all_assets()
        .await
        .map_err(|e| e.to_string())?;

    app.emit_all("market:sync-complete", result.count)?;

    Ok(result)
}

#[tauri::command]
pub async fn get_latest_quotes(
    state: tauri::State``'_, Arc`ServiceContext>`>,
    asset_ids: Vec`i32>`,
) -> Result`Vec``Quote>`, String> {
    state.market_data_service.get_latest_quotes(asset_ids)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_symbol(
    state: tauri::State``'_, Arc`ServiceContext>`>,
    query: String,
) -> Result`Vec``Asset>`, String> {
    state.market_data_service.search_symbols(&query)
        .await
        .map_err(|e| e.to_string())
}
```

---

## Plugin Integration

### File System Plugin

```rust
use tauri_plugin_fs::FsExt;

#[tauri::command]
pub async fn export_data(
    state: tauri::State``'_, Arc`ServiceContext>`>,
    app: AppHandle,
    format: ExportFormat,
) -> Result`String`, String> {
    // 1. Generate export data
    let data = match format {
        ExportFormat::Json => state.export_json().await?,
        ExportFormat::Csv => state.export_csv().await?,
    };

    // 2. Save to file
    let save_path = app.path().app_data_dir()?
        .join(format!("wealthfolio_export.{}", format.extension()));

    FsExt::write_file(&save_path, data)?;

    Ok(save_path.to_string_lossy().to_string())
}
```

### Dialog Plugin

```rust
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

#[tauri::command]
pub async fn confirm_delete_account(
    app: AppHandle,
    account_name: String,
) -> Result`bool`, String> {
    let confirmed = app.dialog()
        .confirm(
            &format!("Are you sure you want to delete '{}'?", account_name),
            MessageDialogKind::Warning,
        )?
        .recv()
        .await;

    Ok(confirmed.is_some())
}
```

---

## Configuration

### Tauri Configuration (`tauri.conf.json`)

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Wealthfolio",
  "version": "1.0.0",
  "identifier": "com.wealthfolio.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "Wealthfolio",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false,
        "minWidth": 800,
        "minHeight": 600
      }
    ],
    "security": {
      "csp": "default-src 'self'; script-src 'self'"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "plugins": {
    "fs": {
      "allow": ["**/*"]
    },
    "dialog": {
      "allow": ["confirm", "message", "open", "save"]
    },
    "shell": {
      "allow": ["open"]
    },
    "log": {
      "allow": ["log"]
    },
    "updater": {
      "active": true
    }
  }
}
```

---

## Cross-Platform Considerations

### Platform-Specific Code

```rust
#[cfg(target_os = "windows")]
fn get_data_dir() -> PathBuf {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .expect("APPDATA not set")
}

#[cfg(target_os = "macos")]
fn get_data_dir() -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join("Library/Application Support"))
        .expect("HOME not set")
}

#[cfg(target_os = "linux")]
fn get_data_dir() -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".local/share"))
        .expect("HOME not set")
}
```

---

## Next Steps

- [Core Services](./core-services) - Business logic implementation
- [Database Schema](./database-schema) - SQLite database structure
- [Frontend Architecture](./frontend) - React frontend details
- [Data Flow](../architecture/data-flow) - Request/response patterns
