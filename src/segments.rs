use crate::config::{Config, NamedColor, SegmentConfig, SegmentId};
use crate::context::{GitStatus, StatusContext};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SegmentPiece {
    pub id: SegmentId,
    pub text: String,
    pub color: Option<NamedColor>,
}

pub fn build_segments(cfg: &Config, ctx: &StatusContext) -> Vec<SegmentPiece> {
    cfg.segments
        .iter()
        .filter(|segment| segment.enabled)
        .filter_map(|segment| build_segment(segment, ctx))
        .collect()
}

fn build_segment(segment: &SegmentConfig, ctx: &StatusContext) -> Option<SegmentPiece> {
    let value = match segment.id {
        SegmentId::Model => ctx.model.clone(),
        SegmentId::Cwd => Some(render_cwd(segment, ctx)),
        SegmentId::Git => ctx.git.as_ref().map(render_git),
        SegmentId::Context => render_context(segment, ctx),
        SegmentId::Tokens => render_tokens(ctx),
        SegmentId::Limits => render_limits(ctx),
        SegmentId::Session => ctx
            .session
            .as_ref()
            .and_then(|s| s.thread_id.as_ref())
            .map(|id| format!("{}", shorten_uuid(id))),
        SegmentId::CodexVersion => ctx
            .session
            .as_ref()
            .and_then(|s| s.cli_version.as_ref())
            .map(|v| format!("v{v}")),
    }?;

    let text = if segment.icon.is_empty() {
        value
    } else {
        format!("{} {}", segment.icon, value)
    };

    Some(SegmentPiece {
        id: segment.id,
        text,
        color: segment.color,
    })
}

fn render_cwd(segment: &SegmentConfig, ctx: &StatusContext) -> String {
    let use_basename = segment
        .options
        .get("basename")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if use_basename {
        if let Some(name) = ctx.cwd.file_name().and_then(|n| n.to_str()) {
            return name.to_string();
        }
    }
    ctx.cwd.display().to_string()
}

fn render_git(git: &GitStatus) -> String {
    let mut parts = vec![git.branch.clone()];
    if git.dirty {
        parts.push("*".to_string());
    }
    if let Some(ahead) = git.ahead.filter(|v| *v > 0) {
        parts.push(format!("+{ahead}"));
    }
    if let Some(behind) = git.behind.filter(|v| *v > 0) {
        parts.push(format!("-{behind}"));
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
    let mut parts = Vec::new();
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

fn shorten_uuid(s: &str) -> &str {
    s.get(0..8).unwrap_or(s)
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
}
