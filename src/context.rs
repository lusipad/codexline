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
    pub staged: u32,
    pub unstaged: u32,
    pub untracked: u32,
    pub conflicted: u32,
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
    pub model_provider: Option<String>,
}
