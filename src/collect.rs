use crate::config::{codex_home, Config};
use crate::context::{
    GitStatus, RateLimitSnapshot, SessionMetaSnapshot, StatusContext, TokenUsageSnapshot,
};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use serde_json::Value;
use std::cmp::Reverse;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Collection {
    pub codex_home: PathBuf,
    pub sessions_dir: PathBuf,
    pub latest_rollout: Option<PathBuf>,
    pub context: StatusContext,
}

#[derive(Default)]
struct RolloutInfo {
    path: Option<PathBuf>,
    model: Option<String>,
    usage: Option<TokenUsageSnapshot>,
    limits: Option<RateLimitSnapshot>,
    session: Option<SessionMetaSnapshot>,
}

pub fn collect(cfg: &Config) -> Result<Collection> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let git = collect_git(&cwd);

    let codex_home_dir = codex_home();
    let sessions_dir = cfg
        .rollout
        .path_override
        .clone()
        .unwrap_or_else(|| codex_home_dir.join("sessions"));

    let rollout = collect_rollout(cfg, &sessions_dir)?;

    let context = StatusContext {
        now: Utc::now(),
        cwd: cwd.clone(),
        project_root: get_git_root(&cwd),
        model: rollout.model,
        git,
        usage: rollout.usage,
        limits: rollout.limits,
        session: rollout.session,
    };

    Ok(Collection {
        codex_home: codex_home_dir,
        sessions_dir,
        latest_rollout: rollout.path,
        context,
    })
}

fn collect_git(cwd: &Path) -> Option<GitStatus> {
    let output = run_git(cwd, ["status", "--porcelain=2", "--branch"])?;

    let mut branch = "unknown".to_string();
    let mut staged: u32 = 0;
    let mut unstaged: u32 = 0;
    let mut untracked: u32 = 0;
    let mut conflicted: u32 = 0;
    let mut ahead: Option<i64> = None;
    let mut behind: Option<i64> = None;

    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            branch = rest.trim().to_string();
            continue;
        }

        if let Some(rest) = line.strip_prefix("# branch.ab ") {
            let mut parts = rest.split_whitespace();
            ahead = parts
                .next()
                .and_then(|s| s.strip_prefix("+"))
                .and_then(|s| s.parse::<i64>().ok());
            behind = parts
                .next()
                .and_then(|s| s.strip_prefix("-"))
                .and_then(|s| s.parse::<i64>().ok());
            continue;
        }

        if line.starts_with("1 ") || line.starts_with("2 ") {
            let mut parts = line.split_whitespace();
            let _ = parts.next();
            if let Some(xy) = parts.next() {
                let bytes = xy.as_bytes();
                let x = bytes.first().copied().unwrap_or(46);
                let y = bytes.get(1).copied().unwrap_or(46);
                if x != 46 {
                    staged = staged.saturating_add(1);
                }
                if y != 46 {
                    unstaged = unstaged.saturating_add(1);
                }
            }
            continue;
        }

        if line.starts_with("u ") {
            conflicted = conflicted.saturating_add(1);
            continue;
        }

        if line.starts_with("? ") {
            untracked = untracked.saturating_add(1);
        }
    }

    let dirty = staged + unstaged + untracked + conflicted > 0;

    Some(GitStatus {
        branch,
        dirty,
        staged,
        unstaged,
        untracked,
        conflicted,
        ahead,
        behind,
    })
}

fn get_git_root(cwd: &Path) -> Option<PathBuf> {
    run_git(cwd, ["rev-parse", "--show-toplevel"]).map(|s| PathBuf::from(s.trim()))
}

fn run_git<const N: usize>(cwd: &Path, args: [&str; N]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

fn collect_rollout(cfg: &Config, sessions_dir: &Path) -> Result<RolloutInfo> {
    if !sessions_dir.exists() {
        return Ok(RolloutInfo::default());
    }

    let max_age = Utc::now() - Duration::days(cfg.rollout.scan_depth_days as i64);
    let max_age_system =
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(max_age.timestamp().max(0) as u64);

    let mut files: Vec<(SystemTime, PathBuf)> = WalkDir::new(sessions_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("jsonl"))
        })
        .filter_map(|entry| {
            let meta = entry.metadata().ok()?;
            let modified = meta.modified().ok()?;
            if modified < max_age_system {
                return None;
            }
            Some((modified, entry.into_path()))
        })
        .collect();

    files.sort_by_key(|(mtime, _)| Reverse(*mtime));

    let mut info = RolloutInfo::default();
    for (_, path) in files.into_iter().take(cfg.rollout.max_files) {
        let parsed = parse_rollout_file(&path)?;
        if parsed.model.is_none()
            && parsed.usage.is_none()
            && parsed.limits.is_none()
            && parsed.session.is_none()
        {
            continue;
        }
        info.path = Some(path);
        info.model = parsed.model;
        info.usage = parsed.usage;
        info.limits = parsed.limits;
        info.session = parsed.session;
        break;
    }

    Ok(info)
}

fn parse_rollout_file(path: &Path) -> Result<RolloutInfo> {
    let file = File::open(path)
        .with_context(|| format!("failed to open rollout file: {}", path.display()))?;

    let mut info = RolloutInfo::default();

    for line_result in BufReader::new(file).lines() {
        let line = line_result?;
        let value: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let typ = value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let payload = value.get("payload").unwrap_or(&Value::Null);

        match typ {
            "session_meta" => {
                info.session = Some(SessionMetaSnapshot {
                    thread_id: payload
                        .get("id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    cli_version: payload
                        .get("cli_version")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    model_provider: payload
                        .get("model_provider")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                });

                if info.model.is_none() {
                    info.model = payload
                        .get("model_provider")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned);
                }
            }
            "turn_context" => {
                if info.model.is_none() {
                    info.model = payload
                        .get("model")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned);
                }
            }
            "event_msg" => {
                apply_event_payload(payload, &mut info);
            }
            "token_count" => {
                apply_token_count(payload, &mut info);
            }
            _ => {}
        }
    }

    if info.model.is_none() {
        info.model = info
            .session
            .as_ref()
            .and_then(|session| session.model_provider.clone());
    }

    Ok(info)
}

fn apply_event_payload(payload: &Value, info: &mut RolloutInfo) {
    let event_type = payload
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if event_type != "token_count" {
        return;
    }

    apply_token_count(payload, info);
}

fn apply_token_count(payload: &Value, info: &mut RolloutInfo) {
    let usage_info = payload.get("info").unwrap_or(payload);

    let total = usage_info
        .get("total_token_usage")
        .and_then(|v| v.get("total_tokens"))
        .and_then(Value::as_i64)
        .unwrap_or(0);

    let input = usage_info
        .get("total_token_usage")
        .and_then(|v| v.get("input_tokens"))
        .and_then(Value::as_i64)
        .unwrap_or(0);

    let output = usage_info
        .get("total_token_usage")
        .and_then(|v| v.get("output_tokens"))
        .and_then(Value::as_i64)
        .unwrap_or(0);

    let context_window = usage_info
        .get("model_context_window")
        .and_then(Value::as_i64);
    let used_percent = context_window
        .filter(|v| *v > 0)
        .map(|v| ((total as f64 / v as f64) * 100.0).round() as i64)
        .map(|v| v.clamp(0, 100));

    info.usage = Some(TokenUsageSnapshot {
        input_tokens: input,
        output_tokens: output,
        total_tokens: total,
        model_context_window: context_window,
        used_percent,
        remaining_percent: used_percent.map(|v| 100 - v),
    });

    let primary = payload
        .get("rate_limits")
        .and_then(|v| v.get("primary"))
        .and_then(|v| v.get("used_percent"))
        .and_then(Value::as_f64);

    let secondary = payload
        .get("rate_limits")
        .and_then(|v| v.get("secondary"))
        .and_then(|v| v.get("used_percent"))
        .and_then(Value::as_f64);

    if primary.is_some() || secondary.is_some() {
        info.limits = Some(RateLimitSnapshot {
            primary_used_percent: primary,
            secondary_used_percent: secondary,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parse_rollout_handles_token_count() {
        let dir = TempDir::new().expect("temp dir");
        let file = dir.path().join("sample.jsonl");
        std::fs::write(
            &file,
            [
                r#"{"timestamp":"x","type":"session_meta","payload":{"id":"abc","cli_version":"0.1.0","model_provider":"gpt-5"}}"#,
                r#"{"timestamp":"x","type":"event_msg","payload":{"type":"token_count","info":{"model_context_window":1000,"total_token_usage":{"input_tokens":200,"output_tokens":10,"total_tokens":550}},"rate_limits":{"primary":{"used_percent":30.5}}}}"#,
            ]
            .join("\n"),
        )
        .expect("write");

        let parsed = parse_rollout_file(&file).expect("parse");
        assert_eq!(parsed.model.as_deref(), Some("gpt-5"));
        assert_eq!(
            parsed.session.as_ref().and_then(|s| s.thread_id.as_deref()),
            Some("abc")
        );
        assert_eq!(parsed.usage.as_ref().and_then(|u| u.used_percent), Some(55));
        assert_eq!(
            parsed.limits.as_ref().and_then(|l| l.primary_used_percent),
            Some(30.5)
        );
    }
}
