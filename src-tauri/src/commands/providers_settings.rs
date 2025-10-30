use tauri::State;
use wealthfolio_core::market_data::MarketDataProviderSetting;

use crate::context::ServiceContext; // To access the service
use std::sync::Arc;


use super::error::{CommandResult, CommandError};

#[tauri::command]
pub async fn get_market_data_providers_settings(
    context: State<'_, Arc<ServiceContext>>,
) -> CommandResult<Vec<MarketDataProviderSetting>> {
    Ok(context
        .market_data_service
        .get_provider_settings()
        .await?)
}

#[tauri::command]
pub async fn update_market_data_provider_settings(
    context: State<'_, Arc<ServiceContext>>,
    provider_id: String,
    priority: i32,
    enabled: bool,
) -> CommandResult<MarketDataProviderSetting> {
    context
        .market_data_service
        .update_provider_setting(
            provider_id.clone(),
            wealthfolio_core::market_data::market_data_model::UpdateMarketDataProviderSetting {
                priority: Some(priority),
                enabled: Some(enabled),
            }
        )
        .await?;
    
    // Get updated settings and find the specific provider
    let settings = context
        .market_data_service
        .get_provider_settings()
        .await?;
    
    settings
        .into_iter()
        .find(|s| s.id == provider_id)
        .ok_or_else(|| CommandError::ServiceError("Provider setting not found after update".to_string()))
}
