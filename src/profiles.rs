use crate::config::{self, Config, SegmentId};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enhancement {
    Git,
    Observability,
}

const QUICK_ORDER: [SegmentId; 8] = [
    SegmentId::Model,
    SegmentId::Cwd,
    SegmentId::Git,
    SegmentId::Context,
    SegmentId::Tokens,
    SegmentId::Limits,
    SegmentId::Session,
    SegmentId::CodexVersion,
];

pub fn apply_quick_config(cfg: &mut Config) {
    ensure_all_segments(cfg);
    reorder_segments(cfg, &QUICK_ORDER);

    for segment in &mut cfg.segments {
        segment.enabled = matches!(
            segment.id,
            SegmentId::Model
                | SegmentId::Cwd
                | SegmentId::Git
                | SegmentId::Context
                | SegmentId::Tokens
        );
    }

    set_option_bool(cfg, SegmentId::Cwd, "basename", true);
    set_option_bool(cfg, SegmentId::Git, "detailed", false);
    set_option_string(cfg, SegmentId::Context, "mode", "used");
}

pub fn apply_enhancement(cfg: &mut Config, enhancement: Enhancement) {
    match enhancement {
        Enhancement::Git => {
            ensure_segment(cfg, SegmentId::Git);
            set_enabled(cfg, SegmentId::Git, true);
            set_option_bool(cfg, SegmentId::Git, "detailed", true);
        }
        Enhancement::Observability => {
            for id in [
                SegmentId::Context,
                SegmentId::Tokens,
                SegmentId::Limits,
                SegmentId::Session,
                SegmentId::CodexVersion,
            ] {
                ensure_segment(cfg, id);
                set_enabled(cfg, id, true);
            }
            set_option_string(cfg, SegmentId::Context, "mode", "used");
            reorder_segments(cfg, &QUICK_ORDER);
        }
    }
}

fn ensure_all_segments(cfg: &mut Config) {
    for id in QUICK_ORDER {
        ensure_segment(cfg, id);
    }
}

fn ensure_segment(cfg: &mut Config, id: SegmentId) {
    if cfg.segments.iter().any(|segment| segment.id == id) {
        return;
    }
    cfg.segments.push(config::default_segment_for(id));
}

fn reorder_segments(cfg: &mut Config, order: &[SegmentId]) {
    let mut ordered = Vec::with_capacity(cfg.segments.len());
    for id in order {
        if let Some(index) = cfg.segments.iter().position(|segment| segment.id == *id) {
            ordered.push(cfg.segments.remove(index));
        }
    }
    ordered.append(&mut cfg.segments);
    cfg.segments = ordered;
}

fn set_enabled(cfg: &mut Config, id: SegmentId, enabled: bool) {
    if let Some(segment) = cfg.segments.iter_mut().find(|segment| segment.id == id) {
        segment.enabled = enabled;
    }
}

fn set_option_bool(cfg: &mut Config, id: SegmentId, key: &str, value: bool) {
    if let Some(segment) = cfg.segments.iter_mut().find(|segment| segment.id == id) {
        segment.options.insert(key.to_string(), Value::Bool(value));
    }
}

fn set_option_string(cfg: &mut Config, id: SegmentId, key: &str, value: &str) {
    if let Some(segment) = cfg.segments.iter_mut().find(|segment| segment.id == id) {
        segment
            .options
            .insert(key.to_string(), Value::String(value.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_segment(cfg: &Config, id: SegmentId) -> &crate::config::SegmentConfig {
        cfg.segments
            .iter()
            .find(|segment| segment.id == id)
            .expect("segment should exist")
    }

    #[test]
    fn quick_config_applies_core_layout() {
        let mut cfg = Config::default();
        for segment in &mut cfg.segments {
            segment.enabled = false;
        }

        apply_quick_config(&mut cfg);

        let ids: Vec<SegmentId> = cfg.segments.iter().map(|segment| segment.id).collect();
        assert_eq!(ids, QUICK_ORDER);

        assert!(get_segment(&cfg, SegmentId::Model).enabled);
        assert!(get_segment(&cfg, SegmentId::Cwd).enabled);
        assert!(get_segment(&cfg, SegmentId::Git).enabled);
        assert!(get_segment(&cfg, SegmentId::Context).enabled);
        assert!(get_segment(&cfg, SegmentId::Tokens).enabled);
        assert!(!get_segment(&cfg, SegmentId::Limits).enabled);
        assert!(!get_segment(&cfg, SegmentId::Session).enabled);
        assert!(!get_segment(&cfg, SegmentId::CodexVersion).enabled);

        assert_eq!(
            get_segment(&cfg, SegmentId::Git)
                .options
                .get("detailed")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            get_segment(&cfg, SegmentId::Context)
                .options
                .get("mode")
                .and_then(|value| value.as_str()),
            Some("used")
        );
    }

    #[test]
    fn git_enhancement_enables_detailed_status() {
        let mut cfg = Config::default();
        let git = cfg
            .segments
            .iter_mut()
            .find(|segment| segment.id == SegmentId::Git)
            .expect("git segment should exist");
        git.enabled = false;
        git.options
            .insert("detailed".to_string(), Value::Bool(false));

        apply_enhancement(&mut cfg, Enhancement::Git);

        assert!(get_segment(&cfg, SegmentId::Git).enabled);
        assert_eq!(
            get_segment(&cfg, SegmentId::Git)
                .options
                .get("detailed")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn observability_enhancement_enables_extra_segments() {
        let mut cfg = Config::default();
        apply_quick_config(&mut cfg);

        apply_enhancement(&mut cfg, Enhancement::Observability);

        assert!(get_segment(&cfg, SegmentId::Context).enabled);
        assert!(get_segment(&cfg, SegmentId::Tokens).enabled);
        assert!(get_segment(&cfg, SegmentId::Limits).enabled);
        assert!(get_segment(&cfg, SegmentId::Session).enabled);
        assert!(get_segment(&cfg, SegmentId::CodexVersion).enabled);
    }
}
