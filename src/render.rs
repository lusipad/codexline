use crate::config::{Config, NamedColor};
use crate::segments::SegmentPiece;

pub fn render_line(cfg: &Config, segments: &[SegmentPiece], plain: bool) -> String {
    let rendered: Vec<String> = segments
        .iter()
        .map(|segment| {
            if plain {
                segment.plain_text()
            } else {
                render_segment(segment)
            }
        })
        .collect();
    rendered.join(&cfg.style.separator)
}

fn render_segment(segment: &SegmentPiece) -> String {
    let mut out = String::new();

    if !segment.icon.is_empty() {
        out.push_str(&paint(&segment.icon, segment.icon_color, segment.bold));
        out.push_str(" ");
    }
    out.push_str(&paint(&segment.value, segment.text_color, segment.bold));

    out
}

fn paint(text: &str, color: Option<NamedColor>, bold: bool) -> String {
    let mut codes: Vec<String> = Vec::new();
    if bold {
        codes.push("1".to_string());
    }
    if let Some(color_code) = color.map(color_code) {
        codes.push(color_code);
    }

    if codes.is_empty() {
        return text.to_string();
    }

    format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text)
}

fn color_code(color: NamedColor) -> String {
    match color {
        NamedColor::Black => "30",
        NamedColor::Red => "31",
        NamedColor::Green => "32",
        NamedColor::Yellow => "33",
        NamedColor::Blue => "34",
        NamedColor::Magenta => "35",
        NamedColor::Cyan => "36",
        NamedColor::White => "37",
        NamedColor::BrightBlack => "90",
        NamedColor::BrightRed => "91",
        NamedColor::BrightGreen => "92",
        NamedColor::BrightYellow => "93",
        NamedColor::BrightBlue => "94",
        NamedColor::BrightMagenta => "95",
        NamedColor::BrightCyan => "96",
        NamedColor::BrightWhite => "97",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SegmentId, StyleConfig, StyleMode};

    #[test]
    fn render_line_without_trailing_separator() {
        let cfg = Config {
            style: StyleConfig {
                mode: StyleMode::Plain,
                separator: " | ".to_string(),
            },
            ..Config::default()
        };

        let segments = vec![
            SegmentPiece {
                id: SegmentId::Model,
                icon: "M".to_string(),
                value: "gpt-5".to_string(),
                icon_color: None,
                text_color: None,
                bold: false,
            },
            SegmentPiece {
                id: SegmentId::Git,
                icon: "GIT".to_string(),
                value: "main".to_string(),
                icon_color: None,
                text_color: None,
                bold: false,
            },
        ];

        assert_eq!(render_line(&cfg, &segments, true), "M gpt-5 | GIT main");
    }
}
