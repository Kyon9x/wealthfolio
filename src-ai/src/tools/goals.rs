//! Goals tool - fetch investment goals with progress.

use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{MAX_GOALS, ToolContext, ToolResult, ToolWithContext};
use crate::error::AiError;

// ============================================================================
// Tool Arguments and Output
// ============================================================================

/// Arguments for the get_goals tool (no required args).
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetGoalsArgs {}

/// DTO for goal data in tool output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalDto {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub target_amount: f64,
    pub current_amount: f64,
    pub progress_percent: f64,
    pub deadline: Option<String>,
    pub is_achieved: bool,
}

/// Output envelope for goals tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetGoalsOutput {
    pub goals: Vec<GoalDto>,
    pub count: usize,
    pub total_target: f64,
    pub total_current: f64,
    pub achieved_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_count: Option<usize>,
}

// ============================================================================
// Tool Implementation
// ============================================================================

/// Tool to get investment goals with progress.
pub struct GetGoalsTool;

#[async_trait]
impl<C: ToolContext + Send + Sync> ToolWithContext<C> for GetGoalsTool {
    fn definition(&self) -> super::ToolDefinition {
        super::ToolDefinition::new(
            "get_goals",
            "Get investment goals with current progress. Returns goal title, target amount, current amount, progress percentage, and deadline for each goal.",
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

        // Fetch goals
        let goals = context
            .goal_service()
            .get_goals()
            .map_err(|e| AiError::ToolExecutionFailed(e.to_string()))?;

        // Fetch allocations for progress
        let allocations = context
            .goal_service()
            .load_goals_allocations()
            .unwrap_or_default();

        // Fetch latest valuations for progress calculation
        let account_ids: Vec<String> = allocations.iter().map(|a| a.account_id.clone()).collect();

        let valuations = context
            .valuation_service()
            .get_latest_valuations(&account_ids)
            .unwrap_or_default();

        // Build valuation lookup: account_id -> total_value in base currency (Decimal)
        let valuation_map: HashMap<String, Decimal> = valuations
            .iter()
            .map(|v| {
                let value_in_base = v.total_value * v.fx_rate_to_base;
                (v.account_id.clone(), value_in_base)
            })
            .collect();

        let original_count = goals.len();

        // Convert to DTOs with progress
        let goals_dto: Vec<GoalDto> = goals
            .into_iter()
            .take(MAX_GOALS)
            .map(|g| {
                // Calculate current amount using percent_allocation per account
                let current_amount_dec: Decimal = allocations
                    .iter()
                    .filter(|a| a.goal_id == g.id)
                    .map(|a| {
                        let account_value = valuation_map
                            .get(&a.account_id)
                            .copied()
                            .unwrap_or(Decimal::ZERO);
                        account_value * Decimal::from(a.percent_allocation) / Decimal::from(100)
                    })
                    .sum();

                let current_amount = current_amount_dec.to_f64().unwrap_or(0.0);

                let progress_percent = if g.target_amount > 0.0 {
                    current_amount / g.target_amount * 100.0
                } else {
                    0.0
                };

                GoalDto {
                    id: g.id,
                    title: g.title,
                    description: g.description,
                    target_amount: g.target_amount,
                    current_amount,
                    progress_percent,
                    deadline: None, // Goal model doesn't have deadline field
                    is_achieved: g.is_achieved,
                }
            })
            .collect();

        let returned_count = goals_dto.len();
        let truncated = original_count > returned_count;

        // Calculate totals
        let total_target: f64 = goals_dto.iter().map(|g| g.target_amount).sum();
        let total_current: f64 = goals_dto.iter().map(|g| g.current_amount).sum();
        let achieved_count = goals_dto.iter().filter(|g| g.is_achieved).count();

        let output = GetGoalsOutput {
            goals: goals_dto,
            count: returned_count,
            total_target,
            total_current,
            achieved_count,
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
    fn test_get_goals_args_default() {
        let args = GetGoalsArgs::default();
        let _ = args;
    }

    #[test]
    fn test_goal_dto_serialization() {
        let dto = GoalDto {
            id: "goal-1".to_string(),
            title: "Retirement".to_string(),
            description: Some("Save for retirement".to_string()),
            target_amount: 1000000.0,
            current_amount: 250000.0,
            progress_percent: 25.0,
            deadline: Some("2040-12-31".to_string()),
            is_achieved: false,
        };

        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("\"title\":\"Retirement\""));
        assert!(json.contains("\"progressPercent\":25"));
    }

    #[test]
    fn test_goals_output_serialization() {
        let output = GetGoalsOutput {
            goals: vec![],
            count: 0,
            total_target: 1000000.0,
            total_current: 250000.0,
            achieved_count: 0,
            truncated: None,
            original_count: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"totalTarget\":1000000"));
    }
}
