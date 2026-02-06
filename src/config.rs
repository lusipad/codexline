use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_separator")]
    pub separator: String,
    #[serde(default)]
    pub rollout: RolloutConfig,
    #[serde(default = "default_segments")]
    pub segments: Vec<SegmentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutConfig {
    #[serde(default = "default_scan_depth_days")]
    pub scan_depth_days: u32,
    #[serde(default = "default_max_files")]
    pub max_files: usize,
    #[serde(default)]
    pub path_override: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentConfig {
    pub id: SegmentId,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub color: Option<NamedColor>,
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentId {
    Model,
    Cwd,
    Git,
    Context,
    Tokens,
    Limits,
    Session,
    CodexVersion,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedColor {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitResult {
    Created,
    AlreadyExists,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            separator: default_separator(),
            rollout: RolloutConfig::default(),
            segments: default_segments(),
        }
    }
}

impl Default for RolloutConfig {
    fn default() -> Self {
        Self {
            scan_depth_days: default_scan_depth_days(),
            max_files: default_max_files(),
            path_override: None,
        }
    }
}

pub fn config_dir() -> PathBuf {
    codex_home().join("codexline")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn codex_home() -> PathBuf {
    if let Some(path) = std::env::var_os("CODEX_HOME") {
        return PathBuf::from(path);
    }
    match dirs::home_dir() {
        Some(home) => home.join(".codex"),
        None => PathBuf::from(".codex"),
    }
}

pub fn load() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    load_from_path(&path)
}

pub fn load_from_path(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let cfg: Config = toml::from_str(&content)
        .with_context(|| format!("failed to parse config: {}", path.display()))?;
    cfg.validate()?;
    Ok(cfg)
}

pub fn init() -> Result<InitResult> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create dir: {}", parent.display()))?;
    }
    if path.exists() {
        return Ok(InitResult::AlreadyExists);
    }
    let cfg = Config::default();
    save_to_path(&cfg, &path)?;
    Ok(InitResult::Created)
}

pub fn save_to_path(cfg: &Config, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create dir: {}", parent.display()))?;
    }
    let text = toml::to_string_pretty(cfg).context("failed to serialize config")?;
    fs::write(path, text).with_context(|| format!("failed to write config: {}", path.display()))
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.segments.is_empty() {
            bail!("segments cannot be empty");
        }
        let mut seen = HashSet::new();
        for segment in &self.segments {
            if !seen.insert(segment.id) {
                bail!("duplicate segment id: {:?}", segment.id);
            }
        }
        Ok(())
    }
}

pub fn default_segments() -> Vec<SegmentConfig> {
    vec![
        seg(SegmentId::Model, "M", Some(NamedColor::Cyan)),
        seg(SegmentId::Cwd, "DIR", Some(NamedColor::Blue)),
        seg(SegmentId::Git, "GIT", Some(NamedColor::Magenta)),
        seg(SegmentId::Context, "CTX", Some(NamedColor::Yellow)),
        seg(SegmentId::Tokens, "TOK", Some(NamedColor::Green)),
        seg(SegmentId::Limits, "LIM", Some(NamedColor::Red)),
    ]
}

fn seg(id: SegmentId, icon: &str, color: Option<NamedColor>) -> SegmentConfig {
    SegmentConfig {
        id,
        enabled: true,
        icon: icon.to_string(),
        color,
        options: HashMap::new(),
    }
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_separator() -> String {
    " | ".to_string()
}

fn default_scan_depth_days() -> u32 {
    14
}

fn default_max_files() -> usize {
    200
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = Config::default();
        assert!(cfg.validate().is_ok());
    }
}
