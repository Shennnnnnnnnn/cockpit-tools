use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexLocalAccessRoutingStrategy {
    Auto,
    QuotaHighFirst,
    QuotaLowFirst,
    PlanHighFirst,
    PlanLowFirst,
    ExpirySoonFirst,
    Custom,
}

impl Default for CodexLocalAccessRoutingStrategy {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexLocalAccessScope {
    Localhost,
    Lan,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexLocalAccessImageGenerationMode {
    Enabled,
    ImagesOnly,
    Disabled,
}

impl Default for CodexLocalAccessImageGenerationMode {
    fn default() -> Self {
        Self::Enabled
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexLocalAccessRequestKind {
    Text,
    ImageGeneration,
    ImageEdit,
    Other,
}

impl Default for CodexLocalAccessRequestKind {
    fn default() -> Self {
        Self::Other
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexLocalAccessImageGenerationStatus {
    Unknown,
    Available,
    Unavailable,
    Disabled,
}

impl Default for CodexLocalAccessImageGenerationStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

fn default_access_scope_for_existing_config() -> CodexLocalAccessScope {
    CodexLocalAccessScope::Lan
}

fn default_restrict_free_accounts() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessCustomRoutingRule {
    pub account_id: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_custom_routing_weight")]
    pub weight: u32,
}

fn default_custom_routing_weight() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessModelAlias {
    pub source_model: String,
    pub alias: String,
    #[serde(default)]
    pub fork: bool,
}

fn default_session_affinity_ttl_ms() -> i64 {
    60 * 60 * 1000
}

fn default_max_retry_interval_ms() -> u64 {
    3 * 1000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessApiKey {
    pub id: String,
    pub label: String,
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_prefix: Option<String>,
    #[serde(default)]
    pub allowed_models: Vec<String>,
    #[serde(default)]
    pub excluded_models: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<i64>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessCollection {
    pub enabled: bool,
    pub port: u16,
    pub api_key: String,
    #[serde(default)]
    pub api_keys: Vec<CodexLocalAccessApiKey>,
    #[serde(default = "default_access_scope_for_existing_config")]
    pub access_scope: CodexLocalAccessScope,
    #[serde(default)]
    pub image_generation_mode: CodexLocalAccessImageGenerationMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_proxy_url: Option<String>,
    #[serde(default)]
    pub routing_strategy: CodexLocalAccessRoutingStrategy,
    #[serde(default)]
    pub custom_routing_rules: Vec<CodexLocalAccessCustomRoutingRule>,
    #[serde(default)]
    pub model_aliases: Vec<CodexLocalAccessModelAlias>,
    #[serde(default)]
    pub excluded_models: Vec<String>,
    #[serde(default)]
    pub session_affinity: bool,
    #[serde(default = "default_session_affinity_ttl_ms")]
    pub session_affinity_ttl_ms: i64,
    #[serde(default)]
    pub max_retry_credentials: u16,
    #[serde(default = "default_max_retry_interval_ms")]
    pub max_retry_interval_ms: u64,
    #[serde(default)]
    pub disable_cooling: bool,
    #[serde(default = "default_restrict_free_accounts")]
    pub restrict_free_accounts: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound_oauth_account_id: Option<String>,
    pub account_ids: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessUsageStats {
    #[serde(default)]
    pub request_count: u64,
    #[serde(default)]
    pub success_count: u64,
    #[serde(default)]
    pub failure_count: u64,
    #[serde(default)]
    pub total_latency_ms: u64,
    #[serde(default)]
    pub text_request_count: u64,
    #[serde(default)]
    pub image_request_count: u64,
    #[serde(default)]
    pub image_generation_request_count: u64,
    #[serde(default)]
    pub image_edit_request_count: u64,
    #[serde(default)]
    pub image_generation_capability_failure_count: u64,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub cached_tokens: u64,
    #[serde(default)]
    pub reasoning_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessAccountStats {
    pub account_id: String,
    pub email: String,
    #[serde(default)]
    pub usage: CodexLocalAccessUsageStats,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessModelStats {
    pub model_id: String,
    #[serde(default)]
    pub usage: CodexLocalAccessUsageStats,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessApiKeyStats {
    pub api_key_id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub usage: CodexLocalAccessUsageStats,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessStatsWindow {
    #[serde(default)]
    pub since: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub totals: CodexLocalAccessUsageStats,
    #[serde(default)]
    pub accounts: Vec<CodexLocalAccessAccountStats>,
    #[serde(default)]
    pub models: Vec<CodexLocalAccessModelStats>,
    #[serde(default)]
    pub api_keys: Vec<CodexLocalAccessApiKeyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessUsageEvent {
    #[serde(default)]
    pub timestamp: i64,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub api_key_id: String,
    #[serde(default)]
    pub api_key_label: String,
    #[serde(default)]
    pub model_id: String,
    #[serde(default)]
    pub request_kind: CodexLocalAccessRequestKind,
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub error_category: String,
    #[serde(default)]
    pub latency_ms: u64,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub cached_tokens: u64,
    #[serde(default)]
    pub reasoning_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessStats {
    #[serde(default)]
    pub since: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub totals: CodexLocalAccessUsageStats,
    #[serde(default)]
    pub accounts: Vec<CodexLocalAccessAccountStats>,
    #[serde(default)]
    pub models: Vec<CodexLocalAccessModelStats>,
    #[serde(default)]
    pub api_keys: Vec<CodexLocalAccessApiKeyStats>,
    #[serde(default)]
    pub daily: CodexLocalAccessStatsWindow,
    #[serde(default)]
    pub weekly: CodexLocalAccessStatsWindow,
    #[serde(default)]
    pub monthly: CodexLocalAccessStatsWindow,
    #[serde(default)]
    pub events: Vec<CodexLocalAccessUsageEvent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessUsageEventPage {
    pub events: Vec<CodexLocalAccessUsageEvent>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessAccountCooldown {
    pub model_id: String,
    pub next_retry_at: i64,
    pub remaining_ms: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessAccountHealth {
    pub account_id: String,
    pub email: String,
    pub available: bool,
    pub consecutive_failures: u32,
    pub last_success_at: Option<i64>,
    pub last_failure_at: Option<i64>,
    pub last_failure_status: Option<u16>,
    pub last_failure_category: Option<String>,
    pub last_failure_message: Option<String>,
    pub image_generation_status: CodexLocalAccessImageGenerationStatus,
    pub image_generation_checked_at: Option<i64>,
    pub cooldowns: Vec<CodexLocalAccessAccountCooldown>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessState {
    pub collection: Option<CodexLocalAccessCollection>,
    pub running: bool,
    pub api_port_url: Option<String>,
    pub base_url: Option<String>,
    pub lan_base_url: Option<String>,
    pub model_ids: Vec<String>,
    pub last_error: Option<String>,
    pub member_count: usize,
    pub stats: CodexLocalAccessStats,
    pub account_health: Vec<CodexLocalAccessAccountHealth>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessTestFailure {
    pub title: String,
    pub stage: String,
    pub cause: String,
    pub suggestion: String,
    pub status: Option<u16>,
    pub model_id: Option<String>,
    pub detail: Option<String>,
    pub cli_output: Option<String>,
    pub gateway_output: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessTestResult {
    pub model_id: Option<String>,
    pub latency_ms: Option<u64>,
    pub output: Option<String>,
    pub failure: Option<CodexLocalAccessTestFailure>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexLocalAccessPortCleanupResult {
    pub killed_count: u32,
    pub state: CodexLocalAccessState,
}
