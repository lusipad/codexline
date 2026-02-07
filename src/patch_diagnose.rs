use crate::collect::Collection;
use crate::config::{config_path, Config};
use chrono::Utc;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct PatchDiagnosticReport {
    pub mode: String,
    pub generated_at: String,
    pub summary: String,
    pub checks: Vec<PatchCheck>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchCheck {
    pub name: String,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Ok,
    Warn,
    Fail,
}

pub fn run_patch_diagnostics(_cfg: &Config, collection: &Collection) -> PatchDiagnosticReport {
    let mut checks: Vec<PatchCheck> = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();

    let config = config_path();
    if config.exists() {
        checks.push(ok("config_file", format!("{}", config.display())));
    } else {
        checks.push(warn(
            "config_file",
            format!("not found: {}", config.display()),
        ));
        suggestions.push("Run codexline --init to create a baseline config".to_string());
    }

    if collection.codex_home.exists() {
        checks.push(ok(
            "codex_home",
            format!("{}", collection.codex_home.display()),
        ));
    } else {
        checks.push(fail(
            "codex_home",
            format!("missing: {}", collection.codex_home.display()),
        ));
        suggestions.push("Set CODEX_HOME or create ~/.codex directory".to_string());
    }

    if collection.sessions_dir.exists() {
        checks.push(ok(
            "sessions_dir",
            format!("{}", collection.sessions_dir.display()),
        ));
    } else {
        checks.push(warn(
            "sessions_dir",
            format!("missing: {}", collection.sessions_dir.display()),
        ));
        suggestions.push("Run Codex once so sessions directory is initialized".to_string());
    }

    if let Some(path) = &collection.latest_rollout {
        checks.push(ok("latest_rollout", format!("{}", path.display())));
    } else {
        checks.push(warn("latest_rollout", "no rollout files found".to_string()));
        suggestions.push("Use codexline --inspect rollout to debug rollout parsing".to_string());
    }

    match find_executable("codex") {
        Some(path) => checks.push(ok("codex_binary", format!("{}", path.display()))),
        None => {
            checks.push(warn(
                "codex_binary",
                "codex command not found in PATH".to_string(),
            ));
            suggestions.push("Install Codex CLI or add it to PATH".to_string());
        }
    }

    let writable =
        collection.codex_home.is_dir() && is_dir_writable(&collection.codex_home).unwrap_or(false);
    if writable {
        checks.push(ok(
            "codex_home_writable",
            format!("{}", collection.codex_home.display()),
        ));
    } else {
        checks.push(warn(
            "codex_home_writable",
            format!("not writable: {}", collection.codex_home.display()),
        ));
        suggestions.push("Ensure current user can write under CODEX_HOME".to_string());
    }

    if suggestions.is_empty() {
        suggestions.push("No blocking issues detected".to_string());
    }

    let has_fail = checks.iter().any(|c| matches!(c.status, CheckStatus::Fail));
    let has_warn = checks.iter().any(|c| matches!(c.status, CheckStatus::Warn));
    let summary = if has_fail {
        "Patch mode diagnostics found blocking issues".to_string()
    } else if has_warn {
        "Patch mode diagnostics completed with warnings".to_string()
    } else {
        "Patch mode diagnostics completed successfully".to_string()
    };

    PatchDiagnosticReport {
        mode: "diagnostic_only".to_string(),
        generated_at: Utc::now().to_rfc3339(),
        summary,
        checks,
        suggestions,
    }
}

pub fn render_text(report: &PatchDiagnosticReport) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("Codex Patch Compatibility Diagnostic".to_string());
    lines.push("Mode: diagnostic_only (no files modified)".to_string());
    lines.push(format!("Summary: {}", report.summary));
    lines.push(String::new());
    lines.push("Checks:".to_string());

    for check in &report.checks {
        let mark = match check.status {
            CheckStatus::Ok => "[OK]",
            CheckStatus::Warn => "[WARN]",
            CheckStatus::Fail => "[FAIL]",
        };
        lines.push(format!("{} {} - {}", mark, check.name, check.detail));
    }

    lines.push(String::new());
    lines.push("Suggestions:".to_string());
    for item in &report.suggestions {
        lines.push(format!("- {}", item));
    }

    lines.join("\n")
}

fn ok(name: impl Into<String>, detail: String) -> PatchCheck {
    PatchCheck {
        name: name.into(),
        status: CheckStatus::Ok,
        detail,
    }
}

fn warn(name: impl Into<String>, detail: String) -> PatchCheck {
    PatchCheck {
        name: name.into(),
        status: CheckStatus::Warn,
        detail,
    }
}

fn fail(name: impl Into<String>, detail: String) -> PatchCheck {
    PatchCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        detail,
    }
}

fn find_executable(bin: &str) -> Option<PathBuf> {
    let path_env = env::var_os("PATH")?;
    for dir in env::split_paths(&path_env) {
        let candidate = dir.join(bin);
        if candidate.exists() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let candidate_exe = dir.join(format!("{}.exe", bin));
            if candidate_exe.exists() {
                return Some(candidate_exe);
            }
        }
    }
    None
}

fn is_dir_writable(path: &Path) -> Option<bool> {
    let probe = path.join(".codexline_write_probe");
    let result = fs::write(&probe, "probe").is_ok();
    if result {
        let _ = fs::remove_file(&probe);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect;

    #[test]
    fn diagnostics_mode_is_non_mutating() {
        let cfg = Config::default();
        let collection = collect::collect(&cfg).expect("collect");
        let report = run_patch_diagnostics(&cfg, &collection);
        assert_eq!(report.mode, "diagnostic_only");
    }
}
