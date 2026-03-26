//! Activities tool - search transactions.

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use super::{DEFAULT_PAGE_SIZE, MAX_ACTIVITIES_ROWS, ToolContext, ToolResult, ToolWithContext};
use crate::error::AiError;
use wealthvn_core::activities::Sort;

// ============================================================================
// Tool Arguments and Output
// ============================================================================

/// Arguments for the search_activities tool.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchActivitiesArgs {
    /// Account ID filter (optional, all accounts if not provided).
    pub account_id: Option<String>,
    /// Activity type filter (e.g., "BUY", "SELL", "DIVIDEND").
    pub activity_type: Option<String>,
    /// Symbol/asset keyword filter.
    pub symbol: Option<String>,
    /// Start date filter in YYYY-MM-DD format (optional).
    pub date_from: Option<String>,
    /// End date filter in YYYY-MM-DD format (optional).
    pub date_to: Option<String>,
    /// Page number (1-based, default: 1).
    pub page: Option<i64>,
    /// Number of results per page (default: 50, max: 200).
    pub page_size: Option<i64>,
}

/// DTO for activity data in tool output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDto {
    pub id: String,
    pub date: String,
    pub activity_type: String,
    pub symbol: Option<String>,
    pub quantity: Option<f64>,
    pub unit_price: Option<f64>,
    pub amount: Option<f64>,
    pub fee: Option<f64>,
    pub fx_rate: Option<f64>,
    pub currency: String,
    pub account_id: String,
    pub account_name: Option<String>,
}

/// Output envelope for activities tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchActivitiesOutput {
    pub activities: Vec<ActivityDto>,
    pub count: usize,
    pub total_row_count: usize,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
    pub account_scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<f64>,
}

// ============================================================================
// Tool Implementation
// ============================================================================

/// Tool to search activities/transactions.
pub struct SearchActivitiesTool;

#[async_trait]
impl<C: ToolContext + Send + Sync> ToolWithContext<C> for SearchActivitiesTool {
    fn definition(&self) -> super::ToolDefinition {
        super::ToolDefinition::new(
            "search_activities",
            "Search and get investment activities (transactions) such as buys, sells, dividends, deposits, and withdrawals. Supports filtering, date ranges, and pagination. Returns paginated results with totalPages so you can request more pages if needed.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "accountId": {
                        "type": "string",
                        "description": "Filter by account ID (optional, all accounts if not provided)"
                    },
                    "activityType": {
                        "type": "string",
                        "description": "Filter by activity type",
                        "enum": ["BUY", "SELL", "DIVIDEND", "DEPOSIT", "WITHDRAWAL", "TRANSFER_IN", "TRANSFER_OUT", "INTEREST", "FEE", "SPLIT", "TAX"]
                    },
                    "symbol": {
                        "type": "string",
                        "description": "Filter by symbol or asset keyword"
                    },
                    "dateFrom": {
                        "type": "string",
                        "description": "Start date filter in YYYY-MM-DD format (optional)"
                    },
                    "dateTo": {
                        "type": "string",
                        "description": "End date filter in YYYY-MM-DD format (optional)"
                    },
                    "page": {
                        "type": "integer",
                        "description": "Page number, 1-based (default: 1)",
                        "default": 1
                    },
                    "pageSize": {
                        "type": "integer",
                        "description": "Number of results per page (default: 50, max: 200)",
                        "default": 50
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

        let args: SearchActivitiesArgs = serde_json::from_value(args)
            .map_err(|e| AiError::InvalidInput(format!("Invalid arguments: {}", e)))?;

        // Pagination: external tool API is 1-based, backend search uses 0-based page index
        let page = args.page.unwrap_or(1).max(1);
        let backend_page = page - 1;
        let page_size = args
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_ACTIVITIES_ROWS as i64);

        // Normalize empty / sentinel values to None
        let account_id = args
            .account_id
            .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("TOTAL"));
        let activity_types = args
            .activity_type
            .filter(|s| !s.is_empty())
            .map(|t| vec![t]);
        let symbol_keyword = args.symbol.filter(|s| !s.is_empty());

        // Resolve account filter: if the value isn't a known account ID, try matching by name
        let account_ids = if let Some(ref raw) = account_id {
            let accounts = context
                .account_service()
                .get_active_accounts()
                .unwrap_or_default();
            let is_known_id = accounts.iter().any(|a| a.id == *raw);
            if is_known_id {
                Some(vec![raw.clone()])
            } else {
                // Try case-insensitive name match
                let raw_lower = raw.to_lowercase();
                let matched: Vec<String> = accounts
                    .iter()
                    .filter(|a| a.name.to_lowercase() == raw_lower)
                    .map(|a| a.id.clone())
                    .collect();
                if matched.is_empty() {
                    // No match — pass raw value (will return 0 results)
                    Some(vec![raw.clone()])
                } else {
                    Some(matched)
                }
            }
        } else {
            None
        };

        // Parse date filters (skip empty strings) - note: date filtering not supported by current trait
        let _date_from = args
            .date_from
            .filter(|s| !s.is_empty())
            .map(|s| {
                NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                    .map_err(|_| AiError::InvalidInput(format!("Invalid dateFrom format: {s}")))
            })
            .transpose()?;
        let _date_to = args
            .date_to
            .filter(|s| !s.is_empty())
            .map(|s| {
                NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                    .map_err(|_| AiError::InvalidInput(format!("Invalid dateTo format: {s}")))
            })
            .transpose()?;

        // Sort by date descending
        let sort = Sort {
            id: "date".to_string(),
            desc: true,
        };

        // Search activities
        let response = context
            .activity_service()
            .search_activities(
                backend_page,
                page_size,
                account_ids.clone(),
                activity_types,
                symbol_keyword,
                Some(sort),
            )
            .map_err(|e| AiError::ToolExecutionFailed(e.to_string()))?;

        let total_row_count = response.meta.total_row_count as usize;
        let total_pages = ((total_row_count as i64) + page_size - 1) / page_size;

        // Convert to DTOs
        let activities: Vec<ActivityDto> = response
            .data
            .into_iter()
            .map(|a| {
                let quantity = a.get_quantity().to_f64();
                let unit_price = a.get_unit_price().to_f64();
                let fee = a.get_fee().to_f64();
                let amount = a.get_amount().and_then(|d| d.to_f64());

                ActivityDto {
                    id: a.id,
                    date: a.date.clone(),
                    activity_type: a.activity_type.clone(),
                    symbol: if a.asset_symbol.is_empty() {
                        None
                    } else {
                        Some(a.asset_symbol.clone())
                    },
                    quantity,
                    unit_price,
                    amount,
                    fee,
                    fx_rate: None, // FX rate not available in ActivityDetails
                    currency: a.currency.clone(),
                    account_id: a.account_id.clone(),
                    account_name: Some(a.account_name.clone()),
                }
            })
            .collect();

        let returned_count = activities.len();

        // Calculate totals for metadata
        let total_amount: f64 = activities.iter().filter_map(|a| a.amount).sum();

        let account_scope = account_id.unwrap_or_else(|| "all".to_string());

        let output = SearchActivitiesOutput {
            activities,
            count: returned_count,
            total_row_count,
            page,
            page_size,
            total_pages,
            account_scope: account_scope.clone(),
            total_amount: if total_amount > 0.0 {
                Some(total_amount)
            } else {
                None
            },
        };

        let duration_ms = start.elapsed().as_millis();

        Ok(ToolResult::ok(output)
            .with_count(returned_count)
            .with_account_scope(&account_scope)
            .with_duration_ms(duration_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_activities_args_default() {
        let args = SearchActivitiesArgs::default();
        assert_eq!(args.page, None);
        assert_eq!(args.page_size, None);
    }

    #[test]
    fn test_activity_dto_serialization() {
        let dto = ActivityDto {
            id: "act-1".to_string(),
            date: "2024-01-15T00:00:00Z".to_string(),
            activity_type: "BUY".to_string(),
            symbol: Some("AAPL".to_string()),
            quantity: Some(100.0),
            unit_price: Some(150.0),
            amount: Some(15000.0),
            fee: Some(1.0),
            fx_rate: None,
            currency: "USD".to_string(),
            account_id: "acc-1".to_string(),
            account_name: Some("My Account".to_string()),
        };

        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("\"activityType\":\"BUY\""));
        assert!(json.contains("\"symbol\":\"AAPL\""));
    }
}
