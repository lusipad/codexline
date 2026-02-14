use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub style: StyleConfig,
    #[serde(default)]
    pub rollout: RolloutConfig,
    #[serde(default)]
    pub diagnostics: DiagnosticsConfig,
    #[serde(default = "default_segments")]
    pub segments: Vec<SegmentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    #[serde(default)]
    pub mode: StyleMode,
    #[serde(default = "default_separator")]
    pub separator: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StyleMode {
    Plain,
    #[default]
    NerdFont,
    Powerline,
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
pub struct DiagnosticsConfig {
    #[serde(default = "default_true")]
    pub warn_once: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentConfig {
    pub id: SegmentId,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub icon: IconConfig,
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub styles: TextStyleConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IconConfig {
    #[serde(default)]
    pub plain: String,
    #[serde(default)]
    pub nerd_font: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColorConfig {
    #[serde(default)]
    pub icon: Option<NamedColor>,
    #[serde(default)]
    pub text: Option<NamedColor>,
    #[serde(default)]
    pub background: Option<NamedColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextStyleConfig {
    #[serde(default)]
    pub text_bold: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
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
            style: StyleConfig::default(),
            rollout: RolloutConfig::default(),
            diagnostics: DiagnosticsConfig::default(),
            segments: default_segments(),
        }
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            mode: StyleMode::NerdFont,
            separator: default_separator(),
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

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            warn_once: default_true(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    codex_home().join("codexline")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn themes_dir() -> PathBuf {
    config_dir().join("themes")
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
    ensure_themes_exist();
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let cfg = load_from_path(&path)?;
    Ok(cfg)
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

    crate::themes::write_builtin_themes_if_missing(&themes_dir())?;

    if path.exists() {
        return Ok(InitResult::AlreadyExists);
    }

    let cfg = Config::default();
    save_to_path(&cfg, &path)?;
    Ok(InitResult::Created)
}

pub fn save(cfg: &Config) -> Result<()> {
    save_to_path(cfg, &config_path())
}

pub fn save_to_path(cfg: &Config, path: &Path) -> Result<()> {
    cfg.validate()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create dir: {}", parent.display()))?;
    }
    let text = toml::to_string_pretty(cfg).context("failed to serialize config")?;
    fs::write(path, text).with_context(|| format!("failed to write config: {}", path.display()))
}

pub fn ensure_themes_exist() {
    let _ = crate::themes::write_builtin_themes_if_missing(&themes_dir());
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

        if self.rollout.max_files == 0 {
            bail!("rollout.max_files must be greater than 0");
        }

        Ok(())
    }
}

pub fn default_segments() -> Vec<SegmentConfig> {
    vec![
        segment(
            SegmentId::Model,
            true,
            icon("M", "󰭹"),
            colors(Some(NamedColor::Cyan), Some(NamedColor::BrightCyan)),
        ),
        segment(
            SegmentId::Cwd,
            true,
            icon("DIR", ""),
            colors(Some(NamedColor::Blue), Some(NamedColor::BrightBlue)),
        ),
        segment(
            SegmentId::Git,
            true,
            icon("GIT", ""),
            colors(Some(NamedColor::Magenta), Some(NamedColor::BrightMagenta)),
        ),
        segment(
            SegmentId::Context,
            true,
            icon("CTX", "󰘦"),
            colors(Some(NamedColor::Yellow), Some(NamedColor::BrightYellow)),
        ),
        segment(
            SegmentId::Tokens,
            true,
            icon("TOK", "󰆧"),
            colors(Some(NamedColor::Green), Some(NamedColor::BrightGreen)),
        ),
        segment(
            SegmentId::Limits,
            true,
            icon("LIM", "󰾅"),
            colors(Some(NamedColor::Red), Some(NamedColor::BrightRed)),
        ),
        segment(
            SegmentId::Session,
            false,
            icon("SID", "󱂬"),
            colors(Some(NamedColor::White), Some(NamedColor::BrightWhite)),
        ),
        segment(
            SegmentId::CodexVersion,
            false,
            icon("VER", "󰀘"),
            colors(Some(NamedColor::BrightBlack), Some(NamedColor::White)),
        ),
    ]
}

pub fn default_segment_for(id: SegmentId) -> SegmentConfig {
    default_segments()
        .into_iter()
        .find(|segment| segment.id == id)
        .expect("default segment must exist for every segment id")
}

fn segment(id: SegmentId, enabled: bool, icon: IconConfig, colors: ColorConfig) -> SegmentConfig {
    SegmentConfig {
        id,
        enabled,
        icon,
        colors,
        styles: TextStyleConfig::default(),
        options: HashMap::new(),
    }
}

fn icon(plain: &str, nerd_font: &str) -> IconConfig {
    IconConfig {
        plain: plain.to_string(),
        nerd_font: nerd_font.to_string(),
    }
}

fn colors(icon_color: Option<NamedColor>, text_color: Option<NamedColor>) -> ColorConfig {
    ColorConfig {
        icon: icon_color,
        text: text_color,
        background: None,
    }
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_separator() -> String {
    " · ".to_string()
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

    #[test]
    fn default_segments_include_all() {
        let cfg = Config::default();
        assert_eq!(cfg.segments.len(), 8);
    }
}
