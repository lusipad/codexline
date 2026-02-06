use crate::config::NamedColor;
use crate::segments::SegmentPiece;

pub fn render_line(segments: &[SegmentPiece], separator: &str, plain: bool) -> String {
    let parts: Vec<String> = segments
        .iter()
        .map(|segment| {
            if plain {
                segment.text.clone()
            } else {
                colorize(&segment.text, segment.color)
            }
        })
        .collect();
    parts.join(separator)
}

fn colorize(text: &str, color: Option<NamedColor>) -> String {
    let Some(code) = color.map(color_code) else {
        return text.to_string();
    };
    format!("\x1b[{code}m{text}\x1b[0m")
}

fn color_code(color: NamedColor) -> String {
    match color {
        NamedColor::Red => "31",
        NamedColor::Green => "32",
        NamedColor::Yellow => "33",
        NamedColor::Blue => "34",
        NamedColor::Magenta => "35",
        NamedColor::Cyan => "36",
        NamedColor::White => "37",
        NamedColor::BrightBlack => "90",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SegmentId;

    #[test]
    fn render_line_without_trailing_separator() {
        let segments = vec![
            SegmentPiece {
                id: SegmentId::Model,
                text: "M gpt-5".to_string(),
                color: None,
            },
            SegmentPiece {
                id: SegmentId::Git,
                text: "GIT main".to_string(),
                color: None,
            },
        ];
        assert_eq!(render_line(&segments, " | ", true), "M gpt-5 | GIT main");
    }
}
