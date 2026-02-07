use crate::config::{Config, NamedColor, SegmentConfig, SegmentId, StyleMode};
use crate::context::{GitStatus, StatusContext};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SegmentPiece {
    pub id: SegmentId,
    pub icon: String,
    pub value: String,
    pub icon_color: Option<NamedColor>,
    pub text_color: Option<NamedColor>,
    pub bold: bool,
}

impl SegmentPiece {
    pub fn plain_text(&self) -> String {
        if self.icon.is_empty() {
            self.value.clone()
        } else {
            format!("{} {}", self.icon, self.value)
        }
    }
}

pub fn build_segments(cfg: &Config, ctx: &StatusContext) -> Vec<SegmentPiece> {
    cfg.segments
        .iter()
        .filter(|segment| segment.enabled)
        .filter_map(|segment| build_segment(cfg.style.mode, segment, ctx))
        .collect()
}

fn build_segment(
    mode: StyleMode,
    segment: &SegmentConfig,
    ctx: &StatusContext,
) -> Option<SegmentPiece> {
    let value = match segment.id {
        SegmentId::Model => ctx.model.as_ref().map(|name| simplify_model_name(name)),
        SegmentId::Cwd => Some(render_cwd(segment, ctx)),
        SegmentId::Git => ctx.git.as_ref().map(|git| render_git(mode, segment, git)),
        SegmentId::Context => render_context(segment, ctx),
        SegmentId::Tokens => render_tokens(ctx),
        SegmentId::Limits => render_limits(ctx),
        SegmentId::Session => ctx
            .session
            .as_ref()
            .and_then(|s| s.thread_id.as_ref())
            .map(|id| shorten_uuid(id).to_string()),
        SegmentId::CodexVersion => ctx
            .session
            .as_ref()
            .and_then(|s| s.cli_version.as_ref())
            .map(|version| format!("v{version}")),
    }?;

    Some(SegmentPiece {
        id: segment.id,
        icon: icon_for_mode(mode, segment),
        value,
        icon_color: segment.colors.icon,
        text_color: segment.colors.text,
        bold: segment.styles.text_bold,
    })
}

fn icon_for_mode(mode: StyleMode, segment: &SegmentConfig) -> String {
    match mode {
        StyleMode::Plain => segment.icon.plain.clone(),
        StyleMode::NerdFont | StyleMode::Powerline => {
            if segment.icon.nerd_font.is_empty() {
                segment.icon.plain.clone()
            } else {
                segment.icon.nerd_font.clone()
            }
        }
    }
}

fn render_cwd(segment: &SegmentConfig, ctx: &StatusContext) -> String {
    let basename = segment
        .options
        .get("basename")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if basename {
        if let Some(name) = ctx.cwd.file_name().and_then(|n| n.to_str()) {
            return name.to_string();
        }
    }
    ctx.cwd.display().to_string()
}

fn render_git(mode: StyleMode, segment: &SegmentConfig, git: &GitStatus) -> String {
    let detailed = segment
        .options
        .get("detailed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let clean_symbol = match mode {
        StyleMode::Plain => "ok",
        StyleMode::NerdFont | StyleMode::Powerline => "✓",
    };
    let dirty_symbol = match mode {
        StyleMode::Plain => "*",
        StyleMode::NerdFont | StyleMode::Powerline => "●",
    };
    let conflict_symbol = match mode {
        StyleMode::Plain => "!",
        StyleMode::NerdFont | StyleMode::Powerline => "⚠",
    };

    let status_symbol = if git.conflicted > 0 {
        conflict_symbol
    } else if git.dirty {
        dirty_symbol
    } else {
        clean_symbol
    };

    let mut parts = vec![git.branch.clone(), status_symbol.to_string()];

    if let Some(v) = git.ahead.filter(|v| *v > 0) {
        parts.push(format!("↑{v}"));
    }
    if let Some(v) = git.behind.filter(|v| *v > 0) {
        parts.push(format!("↓{v}"));
    }

    if detailed {
        if git.staged > 0 {
            parts.push(format!("S{}", git.staged));
        }
        if git.unstaged > 0 {
            parts.push(format!("U{}", git.unstaged));
        }
        if git.untracked > 0 {
            parts.push(format!("N{}", git.untracked));
        }
        if git.conflicted > 0 {
            parts.push(format!("C{}", git.conflicted));
        }
    }

    parts.join(" ")
}

fn render_context(segment: &SegmentConfig, ctx: &StatusContext) -> Option<String> {
    let usage = ctx.usage.as_ref()?;
    let mode = segment
        .options
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("remaining");

    match mode {
        "used" => usage.used_percent.map(|v| format!("{v}% used")),
        _ => usage.remaining_percent.map(|v| format!("{v}% left")),
    }
}

fn render_tokens(ctx: &StatusContext) -> Option<String> {
    let usage = ctx.usage.as_ref()?;
    if usage.total_tokens <= 0 {
        return None;
    }
    Some(format!(
        "{} in {} out {} total",
        compact_tokens(usage.input_tokens),
        compact_tokens(usage.output_tokens),
        compact_tokens(usage.total_tokens)
    ))
}

fn render_limits(ctx: &StatusContext) -> Option<String> {
    let limits = ctx.limits.as_ref()?;
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = limits.primary_used_percent {
        parts.push(format!("5h {}%", v.round() as i64));
    }
    if let Some(v) = limits.secondary_used_percent {
        parts.push(format!("weekly {}%", v.round() as i64));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn simplify_model_name(model: &str) -> String {
    let lower = model.to_lowercase();
    if lower.contains("claude-4-sonnet") || lower.contains("claude-sonnet-4") {
        return "Sonnet 4".to_string();
    }
    if lower.contains("claude-3-7-sonnet") {
        return "Sonnet 3.7".to_string();
    }
    if lower.contains("gpt-5-codex") {
        return "gpt-5-codex".to_string();
    }
    if lower.contains("gpt-5") {
        return "gpt-5".to_string();
    }
    model.to_string()
}

fn shorten_uuid(value: &str) -> &str {
    value.get(0..8).unwrap_or(value)
}

pub fn compact_tokens(value: i64) -> String {
    let abs = value.unsigned_abs() as f64;
    if abs >= 1_000_000.0 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if abs >= 1_000.0 {
        format!("{:.1}K", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_tokens_formats_suffix() {
        assert_eq!(compact_tokens(999), "999");
        assert_eq!(compact_tokens(1200), "1.2K");
        assert_eq!(compact_tokens(2_300_000), "2.3M");
    }

    #[test]
    fn simplify_model_name_maps_known_values() {
        assert_eq!(simplify_model_name("claude-4-sonnet-202501"), "Sonnet 4");
        assert_eq!(simplify_model_name("gpt-5-codex"), "gpt-5-codex");
    }
}
