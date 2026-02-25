use crate::domain::{TaskStatus, TaskType, format_local_human, parse_utc};
use ratatui::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::str::FromStr;

const DIMMED: Style = Style::new().fg(Color::DarkGray);

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, preview_text: &str) {
    let mut section = None;
    let lines: Vec<Line> = preview_text
        .lines()
        .map(|line| {
            if line.starts_with('#') {
                section = None;
                return Line::from(Span::styled(
                    line.to_string(),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            if let Some((key, value)) = try_parse_metadata(line) {
                section = if key == "discard-note" {
                    Some("discard-note")
                } else {
                    None
                };
                let rendered_value = display_metadata_value(key, value);
                let value_style = metadata_value_style(key, value);
                return Line::from(vec![
                    Span::styled(format!("{key}:"), metadata_label_style()),
                    Span::raw(" "),
                    Span::styled(rendered_value, value_style),
                ]);
            }

            if section == Some("discard-note") {
                return Line::from(Span::styled(line.to_string(), Style::default().fg(Color::Red)));
            }

            Line::from(line.to_string())
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Preview"))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn try_parse_metadata(line: &str) -> Option<(&str, &str)> {
    let keys = [
        "status",
        "type",
        "created",
        "updated",
        "discard-note",
        "details",
    ];
    let (key, value) = line.split_once(':')?;
    if keys.contains(&key) {
        return Some((key, value.trim()));
    }
    None
}

fn display_metadata_value(key: &str, value: &str) -> String {
    match key {
        "created" | "updated" => parse_utc(value)
            .map(format_local_human)
            .unwrap_or_else(|_| value.to_string()),
        _ => value.to_string(),
    }
}

fn metadata_label_style() -> Style {
    Style::default().fg(Color::Magenta)
}

fn metadata_value_style(key: &str, value: &str) -> Style {
    match key {
        "status" => match TaskStatus::from_str(value) {
            Ok(TaskStatus::Todo) => DIMMED,
            Ok(TaskStatus::InProgress) => Style::new().fg(Color::Magenta),
            Ok(TaskStatus::Done) => Style::new().fg(Color::Green),
            Ok(TaskStatus::Discard) => Style::new().fg(Color::Red),
            Err(_) => Style::default(),
        },
        "type" => match TaskType::from_str(value) {
            Ok(TaskType::Task) => Style::new().fg(Color::Blue),
            Ok(TaskType::Bug) => Style::new().fg(Color::Red),
            Err(_) => Style::default(),
        },
        "created" | "updated" => DIMMED,
        "discard-note" => Style::new().fg(Color::Red),
        _ => Style::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        display_metadata_value, metadata_label_style, metadata_value_style, try_parse_metadata,
    };
    use ratatui::style::{Color, Modifier};

    #[test]
    fn parses_metadata_keys_and_trims_value() {
        let (key, value) = try_parse_metadata("status: in-progress").unwrap();
        assert_eq!(key, "status");
        assert_eq!(value, "in-progress");

        let (key, value) = try_parse_metadata("discard-note: stale").unwrap();
        assert_eq!(key, "discard-note");
        assert_eq!(value, "stale");
    }

    #[test]
    fn localizes_preview_timestamps() {
        let rendered = display_metadata_value("created", "2026-02-22T06:30:45Z");
        assert!(!rendered.contains('T'));
        assert!(!rendered.ends_with('Z'));
    }

    #[test]
    fn matches_status_and_type_colors_with_table() {
        assert_eq!(metadata_label_style().fg, Some(Color::Magenta));
        assert_eq!(
            metadata_value_style("status", "todo").fg,
            Some(Color::DarkGray)
        );
        assert_eq!(
            metadata_value_style("status", "in-progress").fg,
            Some(Color::Magenta)
        );
        assert_eq!(
            metadata_value_style("status", "done").fg,
            Some(Color::Green)
        );
        assert_eq!(
            metadata_value_style("status", "discard").fg,
            Some(Color::Red)
        );
        assert_eq!(metadata_value_style("type", "task").fg, Some(Color::Blue));
        assert_eq!(metadata_value_style("type", "bug").fg, Some(Color::Red));
        assert_eq!(
            metadata_value_style("discard-note", "outdated").fg,
            Some(Color::Red)
        );
        assert_eq!(
            metadata_value_style("discard-note", "outdated").add_modifier,
            Modifier::empty()
        );
    }
}
