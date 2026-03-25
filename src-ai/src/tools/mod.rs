//! AI assistant tools for portfolio data access.
//!
//! This module provides tool definitions and a registry for AI tools:
//! - Tool trait: Base trait for defining AI tools with metadata
//! - ToolDefinition: Struct for tool metadata (name, description, parameters)
//! - ToolContext: Environment access for services (accounts, holdings, activities, etc.)
//! - ToolResult: Struct for tool outputs with data and metadata
//! - ToolRegistry: Registry for managing available tools
//!
//! Tools are designed to work with the AiEnvironment trait for dependency injection.
//!
//! Key tools:
//! - get_holdings: Fetch portfolio holdings
//! - get_accounts: Fetch active investment accounts
//! - search_activities: Search transactions
//! - get_performance: Fetch portfolio performance metrics
//! - get_goals: Fetch investment goals with progress
//! - get_valuations: Fetch portfolio valuation history

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::AiError;

// ============================================================================
// Constants
// ============================================================================

/// Default page size for activity searches.
pub const DEFAULT_PAGE_SIZE: i64 = 50;

/// Maximum number of activity rows returned per tool call.
pub const MAX_ACTIVITIES_ROWS: usize = 200;

/// Default number of days for valuation history (when no date range specified).
pub const DEFAULT_VALUATIONS_DAYS: i64 = 365;

/// Maximum number of valuation data points returned per tool call.
pub const MAX_VALUATIONS_POINTS: usize = 400;

/// Maximum number of holdings returned per tool call.
pub const MAX_HOLDINGS: usize = 100;

/// Maximum number of goals returned per tool call.
pub const MAX_GOALS: usize = 50;

/// Maximum number of accounts returned per tool call.
pub const MAX_ACCOUNTS: usize = 50;

// ============================================================================
// Tool Definition
// ============================================================================

/// Tool definition metadata for registration.
///
/// Provides the schema information needed for tool registration with LLM providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    /// Unique tool name/identifier.
    pub name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// JSON Schema for tool parameters (properties, required, etc.).
    pub parameters: serde_json::Value,
}

impl ToolDefinition {
    /// Create a new tool definition.
    pub fn new(name: &str, description: &str, parameters: serde_json::Value) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        }
    }

    /// Convert to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

// ============================================================================
// Tool Context
// ============================================================================

/// Environment abstraction for AI tools.
///
/// Provides access to core services for portfolio data access.
/// This trait is implemented by both Tauri and Axum backends.
#[async_trait]
pub trait ToolContext: Send + Sync {
    /// Get the user's base currency (e.g., "USD", "VND").
    fn base_currency(&self) -> String;

    /// Get the account service for fetching accounts.
    fn account_service(&self) -> Arc<dyn wealthvn_core::accounts::AccountServiceTrait>;

    /// Get the activity service for fetching activities.
    fn activity_service(&self) -> Arc<dyn wealthvn_core::activities::ActivityServiceTrait>;

    /// Get the holdings service for fetching holdings.
    fn holdings_service(&self) -> Arc<dyn wealthvn_core::holdings::HoldingsServiceTrait>;

    /// Get the valuation service for fetching valuations.
    fn valuation_service(&self) -> Arc<dyn wealthvn_core::valuation::ValuationServiceTrait>;

    /// Get the goal service for fetching goals.
    fn goal_service(&self) -> Arc<dyn wealthvn_core::goals::GoalServiceTrait>;

    /// Get the performance service for portfolio performance metrics.
    fn performance_service(&self) -> Arc<dyn wealthvn_core::performance::PerformanceServiceTrait>;
}

// ============================================================================
// Tool Result
// ============================================================================

/// Result of tool execution with structured data and metadata.
///
/// All tool outputs are wrapped in this envelope to provide consistent
/// structure for the frontend to render rich UI components.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    /// The result data (structured JSON).
    pub data: serde_json::Value,
    /// Metadata about the result (counts, truncation info, duration, etc.).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, serde_json::Value>,
}

impl ToolResult {
    /// Create a successful result with data.
    pub fn ok(data: impl Serialize) -> Self {
        Self {
            data: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
            meta: HashMap::new(),
        }
    }

    /// Create an empty result.
    pub fn empty() -> Self {
        Self {
            data: serde_json::Value::Null,
            meta: HashMap::new(),
        }
    }

    /// Add metadata to the result.
    pub fn with_meta(mut self, key: &str, value: impl Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.meta.insert(key.to_string(), v);
        }
        self
    }

    /// Add truncation info to metadata.
    pub fn with_truncation(self, original_count: usize, returned_count: usize) -> Self {
        self.with_meta("originalCount", original_count)
            .with_meta("returnedCount", returned_count)
            .with_meta("truncated", original_count > returned_count)
    }

    /// Add duration to metadata.
    pub fn with_duration_ms(self, duration_ms: u128) -> Self {
        self.with_meta("durationMs", duration_ms)
    }

    /// Add account scope to metadata.
    pub fn with_account_scope(self, scope: &str) -> Self {
        self.with_meta("accountScope", scope)
    }

    /// Add row/point count to metadata.
    pub fn with_count(self, count: usize) -> Self {
        self.with_meta("count", count)
    }

    /// Convert to string for sending to LLM.
    pub fn to_llm_string(&self) -> String {
        serde_json::to_string(&self.data).unwrap_or_else(|_| "{}".to_string())
    }
}

// ============================================================================
// Tool Trait
// ============================================================================

/// Base trait for AI tools.
///
/// All tools implement this trait to provide:
/// - Metadata via definition()
/// - Execution logic via call()
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool definition for registration.
    fn definition(&self) -> ToolDefinition;

    /// Execute the tool with given arguments.
    async fn call(&self, args: serde_json::Value) -> Result<ToolResult, AiError>;

    /// Get the tool name.
    fn name(&self) -> String {
        self.definition().name.clone()
    }
}

// ============================================================================
// Tool Registry
// ============================================================================

/// Registry for managing available AI tools.
///
/// The registry stores tools by name and provides methods for
/// registration, lookup, and listing.
#[derive(Clone)]
pub struct ToolRegistry<C: ToolContext> {
    tools: HashMap<String, Arc<dyn ToolWithContext<C>>>,
    context: Arc<C>,
}

impl<C: ToolContext + 'static> ToolRegistry<C> {
    /// Create a new tool registry with the given context.
    pub fn new(context: Arc<C>) -> Self {
        Self {
            tools: HashMap::new(),
            context,
        }
    }

    /// Register a tool in the registry.
    pub fn register<T: ToolWithContext<C> + 'static>(&mut self, tool: T) -> &mut Self {
        let name = tool.name();
        self.tools.insert(name, Arc::new(tool));
        self
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn ToolWithContext<C>>> {
        self.tools.get(name).cloned()
    }

    /// List all registered tool names.
    pub fn list_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get all tool definitions.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Check if a tool is registered.
    pub fn has(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get the context.
    pub fn context(&self) -> Arc<C> {
        self.context.clone()
    }
}

// ============================================================================
// Tool with Context Trait
// ============================================================================

/// Extended tool trait with context access.
///
/// This trait combines Tool with the ability to access the ToolContext.
#[async_trait]
pub trait ToolWithContext<C: ToolContext>: Send + Sync {
    /// Get the tool definition.
    fn definition(&self) -> ToolDefinition;

    /// Get the tool name.
    fn name(&self) -> String {
        self.definition().name.clone()
    }

    /// Execute the tool with context and arguments.
    async fn call(&self, context: &C, args: serde_json::Value) -> Result<ToolResult, AiError>;
}

// ============================================================================
// Helper Macro for Defining Tools
// ============================================================================

/// Macro to simplify tool definition.
///
/// Usage:
/// ```rust
/// define_tool!(
///     MyTool,
///     "my_tool",
///     "Description of what the tool does",
///     MyToolArgs,
///     |context, args| async move {
///         // Tool implementation
///         Ok(ToolResult::empty())
///     }
/// );
/// ```
#[macro_export]
macro_rules! define_tool {
    (
        $tool_name:ident,
        $name:expr,
        $description:expr,
        $args_type:ty,
        $handler:expr
    ) => {
        pub struct $tool_name;

        impl<C: $crate::tools::ToolContext + 'static> $crate::tools::ToolWithContext<C> for $tool_name {
            fn definition(&self) -> $crate::tools::ToolDefinition {
                $crate::tools::ToolDefinition::new(
                    $name,
                    $description,
                    // Generate JSON schema for args
                    serde_json::to_value(<$args_type>::default()).unwrap_or(serde_json::json!({}))
                )
            }

            async fn call(
                &self,
                context: &C,
                args: serde_json::Value,
            ) -> Result<$crate::tools::ToolResult, $crate::error::AiError> {
                // Parse arguments
                let args: $args_type = serde_json::from_value(args)
                    .map_err(|e| $crate::error::AiError::InvalidInput(format!("Invalid arguments: {}", e)))?;

                // Execute handler
                let handler = $handler;
                handler(context, args).await
            }
        }
    };
}

// ============================================================================
// Tool Argument Types
// ============================================================================

/// Arguments for get_accounts tool (no required args).
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountsArgs {}

/// Arguments for get_holdings tool.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetHoldingsArgs {
    /// Account ID, or "TOTAL" for all accounts.
    #[serde(default = "default_account_id")]
    pub account_id: String,
    /// View mode: "table", "treemap", or "both". Default is "treemap".
    #[serde(default = "default_view_mode")]
    pub view_mode: String,
}

fn default_account_id() -> String {
    "TOTAL".to_string()
}

fn default_view_mode() -> String {
    "treemap".to_string()
}

/// Arguments for search_activities tool.
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

/// Arguments for get_performance tool.
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

fn default_period() -> String {
    "YTD".to_string()
}

/// Arguments for get_goals tool (no required args).
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetGoalsArgs {}

/// Arguments for get_valuations tool.
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

// ============================================================================
// Default Tools Allowlist
// ============================================================================

/// Default tools allowed in AI chat.
/// Includes read-only tools and safe mutation tools.
pub const DEFAULT_TOOLS_ALLOWLIST: &[&str] = &[
    "get_holdings",
    "get_accounts",
    "get_performance",
    "search_activities",
    "get_valuations",
    "get_goals",
];

// ============================================================================
// Sub-modules for specific tool implementations
// ============================================================================

// Individual tool implementations
pub mod accounts;
pub mod activities;
pub mod goals;
pub mod holdings;
pub mod performance;
pub mod valuations;

// ============================================================================
// Tool Set Container (for rig-core integration)
// ============================================================================

/// Container for all AI tools, simplifying tool registration across providers.
///
/// This struct holds tool instances that implement rig-core's Tool trait
/// for use with the agent builder. Each tool wraps the corresponding
/// ToolWithContext implementation.
pub struct ToolSet {
    pub holdings: RigHoldingsTool,
    pub accounts: RigAccountsTool,
    pub activities: RigActivitiesTool,
    pub goals: RigGoalsTool,
    pub performance: RigPerformanceTool,
    pub valuation: RigValuationTool,
    pub income: RigIncomeTool,
    pub allocation: RigAllocationTool,
    pub record_activity: RigRecordActivityTool,
    pub record_activities: RigRecordActivitiesTool,
    pub import_csv: RigImportCsvTool,
}

impl ToolSet {
    /// Create a new tool set with all portfolio tools.
    pub fn new(env: Arc<dyn AiEnvironment>, base_currency: String) -> Self {
        Self {
            holdings: RigHoldingsTool::new(env.clone(), base_currency.clone()),
            accounts: RigAccountsTool::new(env.clone()),
            activities: RigActivitiesTool::new(env.clone()),
            goals: RigGoalsTool::new(env.clone()),
            performance: RigPerformanceTool::new(env.clone(), base_currency.clone()),
            valuation: RigValuationTool::new(env.clone(), base_currency.clone()),
            income: RigIncomeTool::new(env.clone()),
            allocation: RigAllocationTool::new(env.clone(), base_currency.clone()),
            record_activity: RigRecordActivityTool::new(env.clone()),
            record_activities: RigRecordActivitiesTool::new(env.clone()),
            import_csv: RigImportCsvTool::new(env, base_currency),
        }
    }
}

// ============================================================================
// Rig-Core Tool Wrappers
// ============================================================================

use rig::{
    completion::ToolDefinition as RigToolDefinition,
    tool::Tool as RigTool,
};
use crate::env::AiEnvironment;

/// Simple error type for rig tool wrappers
#[derive(Debug, thiserror::Error)]
#[error("Tool error: {0}")]
pub struct RigToolError(String);

/// Get holdings tool (stub implementation).
pub struct RigHoldingsTool {
    _private: (),
}

impl RigHoldingsTool {
    pub fn new(_env: Arc<dyn AiEnvironment>, _base_currency: String) -> Self {
        Self { _private: () }
    }
}

impl RigTool for RigHoldingsTool {
    const NAME: &'static str = "get_holdings";

    type Error = RigToolError;
    type Args = super::GetHoldingsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> RigToolDefinition {
        RigToolDefinition {
            name: "get_holdings".to_string(),
            description: "Get portfolio holdings for an account or all accounts. Returns symbol, quantity, market value, cost basis, and gain/loss for each holding.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "account_id": {
                        "type": "string",
                        "description": "Account ID or 'TOTAL' for all accounts",
                        "default": "TOTAL"
                    },
                    "view_mode": {
                        "type": "string",
                        "description": "Display mode: 'treemap', 'table', or 'both'",
                        "default": "treemap"
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Stub: return empty holdings
        Ok(serde_json::json!({
            "holdings": [],
            "total_value": 0,
            "currency": "USD",
            "account_scope": "TOTAL"
        }).to_string())
    }
}

impl Clone for RigHoldingsTool {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}

/// Get accounts tool (stub implementation).
pub struct RigAccountsTool {
    _private: (),
}

impl RigAccountsTool {
    pub fn new(_env: Arc<dyn AiEnvironment>) -> Self {
        Self { _private: () }
    }
}

impl RigTool for RigAccountsTool {
    const NAME: &'static str = "get_accounts";

    type Error = RigToolError;
    type Args = super::GetAccountsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> RigToolDefinition {
        RigToolDefinition {
            name: "get_accounts".to_string(),
            description: "Get active investment accounts".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(serde_json::json!({"accounts": []}).to_string())
    }
}

impl Clone for RigAccountsTool {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}

/// Search activities tool (stub implementation).
pub struct RigActivitiesTool {
    _private: (),
}

impl RigActivitiesTool {
    pub fn new(_env: Arc<dyn AiEnvironment>) -> Self {
        Self { _private: () }
    }
}

impl RigTool for RigActivitiesTool {
    const NAME: &'static str = "search_activities";

    type Error = RigToolError;
    type Args = super::SearchActivitiesArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> RigToolDefinition {
        RigToolDefinition {
            name: "search_activities".to_string(),
            description: "Search transactions and activities".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "account_id": {"type": "string"},
                    "activity_type": {"type": "string"},
                    "symbol": {"type": "string"},
                    "date_from": {"type": "string"},
                    "date_to": {"type": "string"},
                    "page": {"type": "integer"},
                    "page_size": {"type": "integer"}
                },
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(serde_json::json!({
            "activities": [],
            "total_row_count": 0
        }).to_string())
    }
}

impl Clone for RigActivitiesTool {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}

/// Get goals tool (stub implementation).
pub struct RigGoalsTool {
    _private: (),
}

impl RigGoalsTool {
    pub fn new(_env: Arc<dyn AiEnvironment>) -> Self {
        Self { _private: () }
    }
}

impl RigTool for RigGoalsTool {
    const NAME: &'static str = "get_goals";

    type Error = RigToolError;
    type Args = super::GetGoalsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> RigToolDefinition {
        RigToolDefinition {
            name: "get_goals".to_string(),
            description: "Get investment goals with progress".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(serde_json::json!({"goals": []}).to_string())
    }
}

impl Clone for RigGoalsTool {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}

/// Get performance tool (stub implementation).
pub struct RigPerformanceTool {
    _private: (),
}

impl RigPerformanceTool {
    pub fn new(_env: Arc<dyn AiEnvironment>, _base_currency: String) -> Self {
        Self { _private: () }
    }
}

impl RigTool for RigPerformanceTool {
    const NAME: &'static str = "get_performance";

    type Error = RigToolError;
    type Args = super::GetPerformanceArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> RigToolDefinition {
        RigToolDefinition {
            name: "get_performance".to_string(),
            description: "Get portfolio performance metrics".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "account_id": {"type": "string", "default": "TOTAL"},
                    "period": {"type": "string", "default": "YTD"}
                },
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(serde_json::json!({
            "period": "YTD",
            "total_return_pct": 0.0,
            "currency": "USD"
        }).to_string())
    }
}

impl Clone for RigPerformanceTool {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}

/// Get valuation history tool (stub implementation).
pub struct RigValuationTool {
    _private: (),
}

impl RigValuationTool {
    pub fn new(_env: Arc<dyn AiEnvironment>, _base_currency: String) -> Self {
        Self { _private: () }
    }
}

impl RigTool for RigValuationTool {
    const NAME: &'static str = "get_valuation_history";

    type Error = RigToolError;
    type Args = super::GetValuationsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> RigToolDefinition {
        RigToolDefinition {
            name: "get_valuation_history".to_string(),
            description: "Get portfolio valuation history over time".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "account_id": {"type": "string", "default": "TOTAL"},
                    "start_date": {"type": "string"},
                    "end_date": {"type": "string"}
                },
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(serde_json::json!({
            "valuations": [],
            "currency": "USD",
            "account_scope": "TOTAL"
        }).to_string())
    }
}

impl Clone for RigValuationTool {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}

/// Get income tool (stub implementation).
pub struct RigIncomeTool {
    _private: (),
}

impl RigIncomeTool {
    pub fn new(_env: Arc<dyn AiEnvironment>) -> Self {
        Self { _private: () }
    }
}

impl RigTool for RigIncomeTool {
    const NAME: &'static str = "get_income";

    type Error = RigToolError;
    type Args = super::GetAccountsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> RigToolDefinition {
        RigToolDefinition {
            name: "get_income".to_string(),
            description: "Get income summaries (dividends, interest, other)".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(serde_json::json!({"income_summaries": []}).to_string())
    }
}

impl Clone for RigIncomeTool {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}

/// Get asset allocation tool (stub implementation).
pub struct RigAllocationTool {
    _private: (),
}

impl RigAllocationTool {
    pub fn new(_env: Arc<dyn AiEnvironment>, _base_currency: String) -> Self {
        Self { _private: () }
    }
}

impl RigTool for RigAllocationTool {
    const NAME: &'static str = "get_asset_allocation";

    type Error = RigToolError;
    type Args = super::GetHoldingsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> RigToolDefinition {
        RigToolDefinition {
            name: "get_asset_allocation".to_string(),
            description: "Get portfolio allocation breakdown".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "account_id": {"type": "string", "default": "TOTAL"}
                },
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(serde_json::json!({
            "allocation": [],
            "currency": "USD"
        }).to_string())
    }
}

impl Clone for RigAllocationTool {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}

/// Record activity tool (stub implementation).
pub struct RigRecordActivityTool {
    _private: (),
}

impl RigRecordActivityTool {
    pub fn new(_env: Arc<dyn AiEnvironment>) -> Self {
        Self { _private: () }
    }
}

impl RigTool for RigRecordActivityTool {
    const NAME: &'static str = "record_activity";

    type Error = RigToolError;
    type Args = serde_json::Value;
    type Output = String;

    async fn definition(&self, _prompt: String) -> RigToolDefinition {
        RigToolDefinition {
            name: "record_activity".to_string(),
            description: "Create a new activity draft from natural language".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "description": {"type": "string"}
                },
                "required": ["description"]
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(serde_json::json!({
            "activity_draft": null,
            "requires_confirmation": true
        }).to_string())
    }
}

impl Clone for RigRecordActivityTool {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}

/// Record activities tool (stub implementation).
pub struct RigRecordActivitiesTool {
    _private: (),
}

impl RigRecordActivitiesTool {
    pub fn new(_env: Arc<dyn AiEnvironment>) -> Self {
        Self { _private: () }
    }
}

impl RigTool for RigRecordActivitiesTool {
    const NAME: &'static str = "record_activities";

    type Error = RigToolError;
    type Args = serde_json::Value;
    type Output = String;

    async fn definition(&self, _prompt: String) -> RigToolDefinition {
        RigToolDefinition {
            name: "record_activities".to_string(),
            description: "Create multiple activity drafts from natural language".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "description": {"type": "string"}
                },
                "required": ["description"]
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(serde_json::json!({
            "activity_drafts": [],
            "requires_confirmation": true
        }).to_string())
    }
}

impl Clone for RigRecordActivitiesTool {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}

/// Import CSV tool (stub implementation).
pub struct RigImportCsvTool {
    _private: (),
}

impl RigImportCsvTool {
    pub fn new(_env: Arc<dyn AiEnvironment>, _base_currency: String) -> Self {
        Self { _private: () }
    }
}

impl RigTool for RigImportCsvTool {
    const NAME: &'static str = "import_csv";

    type Error = RigToolError;
    type Args = serde_json::Value;
    type Output = String;

    async fn definition(&self, _prompt: String) -> RigToolDefinition {
        RigToolDefinition {
            name: "import_csv".to_string(),
            description: "Import activities from CSV format".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "csv_data": {"type": "string"}
                },
                "required": ["csv_data"]
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(serde_json::json!({
            "import_results": [],
            "requires_confirmation": true
        }).to_string())
    }
}

impl Clone for RigImportCsvTool {
    fn clone(&self) -> Self {
        Self { _private: () }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock context for testing
    struct MockContext;

    impl ToolContext for MockContext {
        fn base_currency(&self) -> String {
            "USD".to_string()
        }

        fn account_service(&self) -> Arc<dyn wealthvn_core::accounts::AccountServiceTrait> {
            unimplemented!()
        }

        fn activity_service(&self) -> Arc<dyn wealthvn_core::activities::ActivityServiceTrait> {
            unimplemented!()
        }

        fn holdings_service(&self) -> Arc<dyn wealthvn_core::holdings::HoldingsServiceTrait> {
            unimplemented!()
        }

        fn valuation_service(&self) -> Arc<dyn wealthvn_core::valuation::ValuationServiceTrait> {
            unimplemented!()
        }

        fn goal_service(&self) -> Arc<dyn wealthvn_core::goals::GoalServiceTrait> {
            unimplemented!()
        }

        fn performance_service(&self) -> Arc<dyn wealthvn_core::performance::PerformanceServiceTrait> {
            unimplemented!()
        }
    }

    #[test]
    fn test_tool_definition() {
        let def = ToolDefinition::new(
            "test_tool",
            "A test tool",
            serde_json::json!({"type": "object", "properties": {}}),
        );

        assert_eq!(def.name, "test_tool");
        assert_eq!(def.description, "A test tool");
    }

    #[test]
    fn test_tool_result() {
        let result = ToolResult::ok(serde_json::json!({"test": "data"}))
            .with_count(10)
            .with_account_scope("TOTAL");

        assert_eq!(result.data["test"], "data");
        assert_eq!(result.meta.get("count").unwrap(), 10);
        assert_eq!(result.meta.get("accountScope").unwrap(), "TOTAL");
    }

    #[test]
    fn test_tool_registry() {
        let context = Arc::new(MockContext);
        let mut registry = ToolRegistry::new(context);

        // After registering tools, the registry should list them
        let names = registry.list_names();
        assert!(names.is_empty());
    }

    #[test]
    fn test_defaults() {
        assert_eq!(default_account_id(), "TOTAL");
        assert_eq!(default_view_mode(), "treemap");
        assert_eq!(default_period(), "YTD");
    }

    #[test]
    fn test_default_args() {
        let args = GetAccountsArgs::default();
        let args2 = SearchActivitiesArgs::default();
        let args3 = GetGoalsArgs::default();

        // Just ensure they compile
        let _ = (args, args2, args3);
    }
}
