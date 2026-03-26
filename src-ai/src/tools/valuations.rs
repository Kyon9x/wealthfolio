//! Valuation history tool - fetch portfolio valuation history.

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{DEFAULT_VALUATIONS_DAYS, MAX_VALUATIONS_POINTS, ToolContext, ToolResult, ToolWithContext};
use crate::error::AiError;

// ============================================================================
// Tool Arguments and Output
// ============================================================================

/// Arguments for the get_valuations tool.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetValuationsArgs {
    /// Account ID, or "TOTAL" for all accounts aggregated.
    #[serde(default = "default_account_id")]
    pub account_id: String,
    /// Start date for the valuation history (YYYY-MM-DD format).
    #[serde(default)]
    pub start_date: Option<String>,
    /// End date for the valuation history (YYYY-MM-DD format).
    #[serde(default)]
    pub end_date: Option<String>,
}

fn default_account_id() -> String {
    "TOTAL".to_string()
}

/// DTO for a single valuation point in tool output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValuationPointDto {
    pub date: String,
    pub total_value: f64,
    pub net_contribution: f64,
    pub currency: String,
}

/// Output envelope for valuation history tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetValuationsOutput {
    pub valuations: Vec<ValuationPointDto>,
    pub account_scope: String,
    pub currency: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_count: Option<usize>,
}

// ============================================================================
// Tool Implementation
// ============================================================================

/// Tool to get portfolio valuation history.
pub struct GetValuationsTool;

#[async_trait]
impl<C: ToolContext + Send + Sync> ToolWithContext<C> for GetValuationsTool {
    fn definition(&self) -> super::ToolDefinition {
        super::ToolDefinition::new(
            "get_valuations",
            "Get historical portfolio valuations over time. Returns daily valuation points with total value and net contributions. Use account_id='TOTAL' for aggregate valuations across all accounts. Useful for analyzing portfolio growth, performance trends, and comparing value vs contributions.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "accountId": {
                        "type": "string",
                        "description": "Account ID to get valuations for, or 'TOTAL' for all accounts aggregated",
                        "default": "TOTAL"
                    },
                    "startDate": {
                        "type": "string",
                        "description": "Start date in YYYY-MM-DD format. Defaults to 365 days ago."
                    },
                    "endDate": {
                        "type": "string",
                        "description": "End date in YYYY-MM-DD format. Defaults to today."
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

        let args: GetValuationsArgs = serde_json::from_value(args)
            .map_err(|e| AiError::InvalidInput(format!("Invalid arguments: {}", e)))?;

        let account_id = &args.account_id;
        let base_currency = context.base_currency();

        // Parse dates with defaults
        let end_date = args
            .end_date
            .as_ref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .unwrap_or_else(|| Utc::now().date_naive());

        let start_date = args
            .start_date
            .as_ref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .unwrap_or_else(|| end_date - chrono::Duration::days(DEFAULT_VALUATIONS_DAYS));

        // Fetch valuations based on account scope
        let valuations = if account_id == "TOTAL" {
            // Get all active accounts and aggregate their valuations
            let accounts = context
                .account_service()
                .get_active_accounts()
                .map_err(|e| AiError::ToolExecutionFailed(e.to_string()))?;

            let mut aggregated: HashMap<NaiveDate, (f64, f64)> = HashMap::new();

            for account in accounts {
                let account_valuations = context
                    .valuation_service()
                    .get_historical_valuations(&account.id, Some(start_date), Some(end_date))
                    .map_err(|e| AiError::ToolExecutionFailed(e.to_string()))?;

                for v in account_valuations {
                    let entry = aggregated.entry(v.valuation_date).or_insert((0.0, 0.0));
                    // Convert to base currency using fx_rate
                    let total_in_base = v.total_value.to_f64().unwrap_or(0.0)
                        * v.fx_rate_to_base.to_f64().unwrap_or(1.0);
                    let contribution_in_base = v.net_contribution.to_f64().unwrap_or(0.0)
                        * v.fx_rate_to_base.to_f64().unwrap_or(1.0);
                    entry.0 += total_in_base;
                    entry.1 += contribution_in_base;
                }
            }

            // Convert aggregated data to sorted vector
            let mut result: Vec<ValuationPointDto> = aggregated
                .into_iter()
                .map(
                    |(date, (total_value, net_contribution))| ValuationPointDto {
                        date: date.format("%Y-%m-%d").to_string(),
                        total_value,
                        net_contribution,
                        currency: base_currency.clone(),
                    },
                )
                .collect();
            result.sort_by(|a, b| a.date.cmp(&b.date));
            result
        } else {
            // Single account valuations
            let account_valuations = context
                .valuation_service()
                .get_historical_valuations(account_id, Some(start_date), Some(end_date))
                .map_err(|e| AiError::ToolExecutionFailed(e.to_string()))?;

            account_valuations
                .into_iter()
                .map(|v| {
                    let total_in_base =
                        v.total_value.to_f64().unwrap_or(0.0) * v.fx_rate_to_base.to_f64().unwrap_or(1.0);
                    let contribution_in_base = v.net_contribution.to_f64().unwrap_or(0.0)
                        * v.fx_rate_to_base.to_f64().unwrap_or(1.0);
                    ValuationPointDto {
                        date: v.valuation_date.format("%Y-%m-%d").to_string(),
                        total_value: total_in_base,
                        net_contribution: contribution_in_base,
                        currency: base_currency.clone(),
                    }
                })
                .collect()
        };

        let original_count = valuations.len();

        // Apply limit
        let valuations: Vec<ValuationPointDto> =
            valuations.into_iter().take(MAX_VALUATIONS_POINTS).collect();

        let returned_count = valuations.len();
        let truncated = original_count > returned_count;

        let output = GetValuationsOutput {
            valuations,
            account_scope: account_id.clone(),
            currency: base_currency,
            start_date: start_date.format("%Y-%m-%d").to_string(),
            end_date: end_date.format("%Y-%m-%d").to_string(),
            truncated: if truncated { Some(true) } else { None },
            original_count: if truncated {
                Some(original_count)
            } else {
                None
            },
        };

        let duration_ms = start.elapsed().as_millis();

        Ok(ToolResult::ok(output)
            .with_count(returned_count)
            .with_truncation(original_count, returned_count)
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
    }

    #[test]
    fn test_valuation_point_dto_serialization() {
        let dto = ValuationPointDto {
            date: "2024-01-15".to_string(),
            total_value: 100000.0,
            net_contribution: 50000.0,
            currency: "USD".to_string(),
        };

        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("\"date\":\"2024-01-15\""));
        assert!(json.contains("\"totalValue\":100000"));
    }

    #[test]
    fn test_valuations_output_serialization() {
        let output = GetValuationsOutput {
            valuations: vec![],
            account_scope: "TOTAL".to_string(),
            currency: "USD".to_string(),
            start_date: "2024-01-01".to_string(),
            end_date: "2024-12-31".to_string(),
            truncated: None,
            original_count: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"accountScope\":\"TOTAL\""));
    }
}
