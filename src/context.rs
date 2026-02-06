use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct StatusContext {
    pub now: DateTime<Utc>,
    pub cwd: PathBuf,
    pub project_root: Option<PathBuf>,
    pub model: Option<String>,
    pub git: Option<GitStatus>,
    pub usage: Option<TokenUsageSnapshot>,
    pub limits: Option<RateLimitSnapshot>,
    pub session: Option<SessionMetaSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitStatus {
    pub branch: String,
    pub dirty: bool,
    pub ahead: Option<i64>,
    pub behind: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenUsageSnapshot {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub model_context_window: Option<i64>,
    pub used_percent: Option<i64>,
    pub remaining_percent: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RateLimitSnapshot {
    pub primary_used_percent: Option<f64>,
    pub secondary_used_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionMetaSnapshot {
    pub thread_id: Option<String>,
    pub cli_version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectReport {
    pub codex_home: PathBuf,
    pub sessions_dir: PathBuf,
    pub latest_rollout: Option<PathBuf>,
    pub context: StatusContext,
}
