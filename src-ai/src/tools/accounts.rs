//! Accounts tool - fetch active accounts.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{MAX_ACCOUNTS, ToolContext, ToolResult, ToolWithContext};
use crate::error::AiError;

// ============================================================================
// Tool Arguments and Output
// ============================================================================

/// Arguments for the get_accounts tool (no required args).
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountsArgs {}

/// DTO for account data in tool output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub is_active: bool,
}

/// Output envelope for accounts tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountsOutput {
    pub accounts: Vec<AccountDto>,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_count: Option<usize>,
}

// ============================================================================
// Tool Implementation
// ============================================================================

/// Tool to get active accounts.
pub struct GetAccountsTool;

#[async_trait]
impl<C: ToolContext + Send + Sync> ToolWithContext<C> for GetAccountsTool {
    fn definition(&self) -> super::ToolDefinition {
        super::ToolDefinition::new(
            "get_accounts",
            "Get the list of active investment accounts. Returns account id, name, type, and currency for each account.",
            serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        )
    }

    async fn call(
        &self,
        context: &C,
        _args: serde_json::Value,
    ) -> Result<ToolResult, AiError> {
        let start = std::time::Instant::now();

        let accounts = context
            .account_service()
            .get_active_accounts()
            .map_err(|e| AiError::ToolExecutionFailed(e.to_string()))?;

        let original_count = accounts.len();
        let accounts_dto: Vec<AccountDto> = accounts
            .into_iter()
            .take(MAX_ACCOUNTS)
            .map(|a| AccountDto {
                id: a.id,
                name: a.name,
                account_type: a.account_type,
                currency: a.currency,
                is_active: a.is_active,
            })
            .collect();

        let returned_count = accounts_dto.len();
        let truncated = original_count > returned_count;

        let output = GetAccountsOutput {
            accounts: accounts_dto,
            count: returned_count,
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
            .with_duration_ms(duration_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_accounts_args_default() {
        let args = GetAccountsArgs::default();
        let _ = args;
    }

    #[test]
    fn test_account_dto_serialization() {
        let dto = AccountDto {
            id: "acc-1".to_string(),
            name: "My Account".to_string(),
            account_type: "BROKERAGE".to_string(),
            currency: "USD".to_string(),
            is_active: true,
        };

        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("\"id\":\"acc-1\""));
        assert!(json.contains("\"name\":\"My Account\""));
    }
}
