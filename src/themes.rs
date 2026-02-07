use crate::config::{
    ColorConfig, Config, IconConfig, NamedColor, SegmentId, StyleConfig, StyleMode,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpec {
    pub name: String,
    #[serde(default)]
    pub style: Option<StyleConfig>,
    #[serde(default)]
    pub segments: Vec<ThemeSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSegment {
    pub id: SegmentId,
    #[serde(default)]
    pub icon: Option<IconConfig>,
    #[serde(default)]
    pub colors: Option<ColorConfig>,
}

pub fn builtin_theme_names() -> Vec<String> {
    vec![
        "default".to_string(),
        "minimal".to_string(),
        "gruvbox".to_string(),
        "nord".to_string(),
        "powerline-dark".to_string(),
        "powerline-light".to_string(),
        "powerline-rose-pine".to_string(),
        "powerline-tokyo-night".to_string(),
    ]
}

pub fn list_theme_names(themes_dir: &Path) -> Result<Vec<String>> {
    let mut names: Vec<String> = builtin_theme_names();

    if themes_dir.exists() {
        for entry in fs::read_dir(themes_dir)
            .with_context(|| format!("failed to read themes dir: {}", themes_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let is_toml = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "toml")
                .unwrap_or(false);
            if !is_toml {
                continue;
            }
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                if !names.iter().any(|v| v == name) {
                    names.push(name.to_string());
                }
            }
        }
    }

    names.sort();
    Ok(names)
}

pub fn write_builtin_themes_if_missing(themes_dir: &Path) -> Result<()> {
    fs::create_dir_all(themes_dir)
        .with_context(|| format!("failed to create themes dir: {}", themes_dir.display()))?;

    for name in builtin_theme_names() {
        let path = themes_dir.join(format!("{}.toml", name));
        if path.exists() {
            continue;
        }
        let Some(theme) = builtin_theme(&name) else {
            continue;
        };
        let text = toml::to_string_pretty(&theme).context("failed to serialize theme")?;
        fs::write(&path, text)
            .with_context(|| format!("failed to write theme file: {}", path.display()))?;
    }

    Ok(())
}

pub fn apply_theme(config: &Config, theme_name: &str, themes_dir: &Path) -> Result<Config> {
    let Some(theme) = load_theme(theme_name, themes_dir)? else {
        bail!("theme not found: {}", theme_name);
    };

    let mut merged = config.clone();
    merged.theme = theme_name.to_string();

    if let Some(style) = theme.style {
        merged.style = style;
    }

    let mut by_id: HashMap<SegmentId, usize> = HashMap::new();
    for (idx, segment) in merged.segments.iter().enumerate() {
        by_id.insert(segment.id, idx);
    }

    for segment_style in theme.segments {
        let Some(index) = by_id.get(&segment_style.id).copied() else {
            continue;
        };
        if let Some(icon) = segment_style.icon {
            merged.segments[index].icon = icon;
        }
        if let Some(colors) = segment_style.colors {
            merged.segments[index].colors = colors;
        }
    }

    Ok(merged)
}

pub fn load_theme(theme_name: &str, themes_dir: &Path) -> Result<Option<ThemeSpec>> {
    if let Some(theme) = builtin_theme(theme_name) {
        return Ok(Some(theme));
    }

    let path = themes_dir.join(format!("{}.toml", theme_name));
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read theme file: {}", path.display()))?;
    let theme: ThemeSpec = toml::from_str(&content)
        .with_context(|| format!("failed to parse theme file: {}", path.display()))?;
    Ok(Some(theme))
}

pub fn builtin_theme(name: &str) -> Option<ThemeSpec> {
    match name {
        "default" => Some(default_theme()),
        "minimal" => Some(minimal_theme()),
        "gruvbox" => Some(gruvbox_theme()),
        "nord" => Some(nord_theme()),
        "powerline-dark" => Some(powerline_dark_theme()),
        "powerline-light" => Some(powerline_light_theme()),
        "powerline-rose-pine" => Some(powerline_rose_pine_theme()),
        "powerline-tokyo-night" => Some(powerline_tokyo_night_theme()),
        _ => None,
    }
}

fn default_theme() -> ThemeSpec {
    ThemeSpec {
        name: "default".to_string(),
        style: Some(StyleConfig {
            mode: StyleMode::NerdFont,
            separator: " · ".to_string(),
        }),
        segments: vec![],
    }
}

fn minimal_theme() -> ThemeSpec {
    ThemeSpec {
        name: "minimal".to_string(),
        style: Some(StyleConfig {
            mode: StyleMode::Plain,
            separator: " | ".to_string(),
        }),
        segments: vec![],
    }
}

fn gruvbox_theme() -> ThemeSpec {
    ThemeSpec {
        name: "gruvbox".to_string(),
        style: Some(StyleConfig {
            mode: StyleMode::NerdFont,
            separator: " ❯ ".to_string(),
        }),
        segments: vec![
            seg_color(SegmentId::Model, NamedColor::BrightYellow),
            seg_color(SegmentId::Cwd, NamedColor::BrightGreen),
            seg_color(SegmentId::Git, NamedColor::BrightRed),
            seg_color(SegmentId::Context, NamedColor::Yellow),
            seg_color(SegmentId::Tokens, NamedColor::Green),
            seg_color(SegmentId::Limits, NamedColor::Red),
        ],
    }
}

fn nord_theme() -> ThemeSpec {
    ThemeSpec {
        name: "nord".to_string(),
        style: Some(StyleConfig {
            mode: StyleMode::NerdFont,
            separator: " • ".to_string(),
        }),
        segments: vec![
            seg_color(SegmentId::Model, NamedColor::Cyan),
            seg_color(SegmentId::Cwd, NamedColor::BrightCyan),
            seg_color(SegmentId::Git, NamedColor::BrightBlue),
            seg_color(SegmentId::Context, NamedColor::BrightWhite),
            seg_color(SegmentId::Tokens, NamedColor::White),
            seg_color(SegmentId::Limits, NamedColor::BrightMagenta),
        ],
    }
}

fn powerline_dark_theme() -> ThemeSpec {
    ThemeSpec {
        name: "powerline-dark".to_string(),
        style: Some(StyleConfig {
            mode: StyleMode::Powerline,
            separator: "  ".to_string(),
        }),
        segments: vec![
            seg_color(SegmentId::Model, NamedColor::BrightWhite),
            seg_color(SegmentId::Cwd, NamedColor::BrightBlue),
            seg_color(SegmentId::Git, NamedColor::BrightMagenta),
            seg_color(SegmentId::Context, NamedColor::BrightYellow),
            seg_color(SegmentId::Tokens, NamedColor::BrightGreen),
            seg_color(SegmentId::Limits, NamedColor::BrightRed),
        ],
    }
}

fn powerline_light_theme() -> ThemeSpec {
    ThemeSpec {
        name: "powerline-light".to_string(),
        style: Some(StyleConfig {
            mode: StyleMode::Powerline,
            separator: "  ".to_string(),
        }),
        segments: vec![
            seg_color(SegmentId::Model, NamedColor::Blue),
            seg_color(SegmentId::Cwd, NamedColor::Cyan),
            seg_color(SegmentId::Git, NamedColor::Magenta),
            seg_color(SegmentId::Context, NamedColor::Yellow),
            seg_color(SegmentId::Tokens, NamedColor::Green),
            seg_color(SegmentId::Limits, NamedColor::Red),
        ],
    }
}

fn powerline_rose_pine_theme() -> ThemeSpec {
    ThemeSpec {
        name: "powerline-rose-pine".to_string(),
        style: Some(StyleConfig {
            mode: StyleMode::Powerline,
            separator: "  ".to_string(),
        }),
        segments: vec![
            seg_color(SegmentId::Model, NamedColor::BrightMagenta),
            seg_color(SegmentId::Cwd, NamedColor::BrightCyan),
            seg_color(SegmentId::Git, NamedColor::BrightYellow),
            seg_color(SegmentId::Context, NamedColor::BrightBlue),
            seg_color(SegmentId::Tokens, NamedColor::BrightGreen),
            seg_color(SegmentId::Limits, NamedColor::BrightRed),
        ],
    }
}

fn powerline_tokyo_night_theme() -> ThemeSpec {
    ThemeSpec {
        name: "powerline-tokyo-night".to_string(),
        style: Some(StyleConfig {
            mode: StyleMode::Powerline,
            separator: "  ".to_string(),
        }),
        segments: vec![
            seg_color(SegmentId::Model, NamedColor::BrightCyan),
            seg_color(SegmentId::Cwd, NamedColor::BrightBlue),
            seg_color(SegmentId::Git, NamedColor::BrightMagenta),
            seg_color(SegmentId::Context, NamedColor::BrightWhite),
            seg_color(SegmentId::Tokens, NamedColor::BrightGreen),
            seg_color(SegmentId::Limits, NamedColor::BrightRed),
        ],
    }
}

fn seg_color(id: SegmentId, text: NamedColor) -> ThemeSegment {
    ThemeSegment {
        id,
        icon: None,
        colors: Some(ColorConfig {
            icon: Some(text),
            text: Some(text),
            background: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn apply_theme_updates_style() {
        let cfg = Config::default();
        let dir = TempDir::new().expect("temp");
        write_builtin_themes_if_missing(dir.path()).expect("write");
        let themed = apply_theme(&cfg, "minimal", dir.path()).expect("apply");
        assert_eq!(themed.style.mode, StyleMode::Plain);
    }
}
