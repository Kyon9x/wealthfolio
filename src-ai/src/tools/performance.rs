//! Performance tool - fetch portfolio performance metrics.

use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use super::{ToolContext, ToolResult, ToolWithContext};
use crate::error::AiError;

// ============================================================================
// Tool Arguments and Output
// ============================================================================

/// Arguments for the get_performance tool.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPerformanceArgs {
    /// Account ID, or "TOTAL" for all accounts.
    #[serde(default = "default_account_id")]
    pub account_id: String,

    /// Period for performance calculation: "1M", "3M", "6M", "YTD", "1Y", "ALL".
    #[serde(default = "default_period")]
    pub period: String,
}

fn default_account_id() -> String {
    "TOTAL".to_string()
}

fn default_period() -> String {
    "YTD".to_string()
}

/// Output for the get_performance tool.
/// Field names match what the frontend expects.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetPerformanceOutput {
    /// Account or portfolio ID.
    pub id: String,
    /// Period start date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_start_date: Option<String>,
    /// Period end date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_end_date: Option<String>,
    /// Base currency.
    pub currency: String,
    /// Cumulative time-weighted return (decimal, e.g., 0.05 = 5%).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cumulative_twr: Option<f64>,
    /// Absolute gain/loss amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gain_loss_amount: Option<f64>,
    /// Annualized TWR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annualized_twr: Option<f64>,
    /// Simple return (decimal).
    pub simple_return: f64,
    /// Annualized simple return.
    pub annualized_simple_return: f64,
    /// Cumulative money-weighted return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cumulative_mwr: Option<f64>,
    /// Annualized MWR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annualized_mwr: Option<f64>,
    /// Portfolio volatility (annualized).
    pub volatility: f64,
    /// Maximum drawdown.
    pub max_drawdown: f64,
}

/// Helper function to convert Decimal to Option<f64>.
fn decimal_to_option_f64(d: Decimal) -> Option<f64> {
    if d.is_zero() {
        Some(0.0)
    } else {
        d.to_f64()
    }
}

// ============================================================================
// Tool Implementation
// ============================================================================

/// Tool to get portfolio performance.
pub struct GetPerformanceTool;

/// Convert a period string to a start date.
fn period_to_start_date(period: &str, end_date: NaiveDate) -> Option<NaiveDate> {
    match period.to_uppercase().as_str() {
        "1M" => Some(end_date - chrono::Duration::days(30)),
        "3M" => Some(end_date - chrono::Duration::days(90)),
        "6M" => Some(end_date - chrono::Duration::days(180)),
        "YTD" => NaiveDate::from_ymd_opt(end_date.year(), 1, 1),
        "1Y" => Some(end_date - chrono::Duration::days(365)),
        _ => None, // None means no start date filter
    }
}

#[async_trait]
impl<C: ToolContext + Send + Sync> ToolWithContext<C> for GetPerformanceTool {
    fn definition(&self) -> super::ToolDefinition {
        super::ToolDefinition::new(
            "get_performance",
            "Get portfolio performance metrics including TWR, MWR, volatility, and max drawdown. Use account_id='TOTAL' for aggregate performance across all accounts.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "accountId": {
                        "type": "string",
                        "description": "Account ID to get performance for, or 'TOTAL' for all accounts",
                        "default": "TOTAL"
                    },
                    "period": {
                        "type": "string",
                        "description": "Time period for performance calculation",
                        "enum": ["1M", "3M", "6M", "YTD", "1Y", "ALL"],
                        "default": "YTD"
                    }
                },
                "required": []
            }),
        )
    }

    async fn call(
        &self,
        context: &C,
        args: serde_json::Value,
    ) -> Result<ToolResult, AiError> {
        let start = std::time::Instant::now();

        let args: GetPerformanceArgs = serde_json::from_value(args)
            .map_err(|e| AiError::InvalidInput(format!("Invalid arguments: {}", e)))?;

        let account_id = &args.account_id;
        let period = args.period.to_uppercase();
        let base_currency = context.base_currency();

        // Calculate date range
        let end_date = Local::now().date_naive();
        let start_date = period_to_start_date(&period, end_date);

        // Use PerformanceService to calculate metrics
        let metrics = context
            .performance_service()
            .calculate_performance_history("account", account_id, start_date, Some(end_date))
            .await
            .map_err(|e| AiError::ToolExecutionFailed(e.to_string()))?;

        let output = GetPerformanceOutput {
            id: metrics.id,
            period_start_date: metrics.period_start_date.map(|d| d.to_string()),
            period_end_date: metrics.period_end_date.map(|d| d.to_string()),
            currency: if metrics.currency.is_empty() {
                base_currency.clone()
            } else {
                metrics.currency
            },
            cumulative_twr: decimal_to_option_f64(metrics.cumulative_twr),
            gain_loss_amount: metrics.gain_loss_amount.and_then(|v| v.to_f64()),
            annualized_twr: decimal_to_option_f64(metrics.annualized_twr),
            simple_return: metrics.simple_return.to_f64().unwrap_or(0.0),
            annualized_simple_return: metrics.annualized_simple_return.to_f64().unwrap_or(0.0),
            cumulative_mwr: decimal_to_option_f64(metrics.cumulative_mwr),
            annualized_mwr: decimal_to_option_f64(metrics.annualized_mwr),
            volatility: metrics.volatility.to_f64().unwrap_or(0.0),
            max_drawdown: metrics.max_drawdown.to_f64().unwrap_or(0.0),
        };

        let duration_ms = start.elapsed().as_millis();

        Ok(ToolResult::ok(output)
            .with_account_scope(account_id)
            .with_duration_ms(duration_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        assert_eq!(default_account_id(), "TOTAL");
        assert_eq!(default_period(), "YTD");
    }

    #[test]
    fn test_period_conversion() {
        let today = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        // Test YTD
        let ytd_start = period_to_start_date("YTD", today);
        assert_eq!(ytd_start, NaiveDate::from_ymd_opt(2024, 1, 1));

        // Test 1M (30 days back)
        let one_month_start = period_to_start_date("1M", today);
        assert_eq!(one_month_start, NaiveDate::from_ymd_opt(2024, 5, 16));

        // Test 1Y (365 days back)
        let one_year_start = period_to_start_date("1Y", today);
        assert_eq!(one_year_start, NaiveDate::from_ymd_opt(2023, 6, 16));

        // Test ALL - returns None (no start date filter)
        let all_start = period_to_start_date("ALL", today);
        assert_eq!(all_start, None);
    }

    #[test]
    fn test_performance_output_serialization() {
        let output = GetPerformanceOutput {
            id: "TOTAL".to_string(),
            period_start_date: Some("2024-01-01".to_string()),
            period_end_date: Some("2024-06-15".to_string()),
            currency: "USD".to_string(),
            cumulative_twr: Some(0.05),
            gain_loss_amount: Some(1000.0),
            annualized_twr: Some(0.10),
            simple_return: 0.08,
            annualized_simple_return: 0.16,
            cumulative_mwr: Some(0.04),
            annualized_mwr: Some(0.08),
            volatility: 0.15,
            max_drawdown: -0.05,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"id\":\"TOTAL\""));
        assert!(json.contains("\"cumulativeTwr\":0.05"));
    }
}
