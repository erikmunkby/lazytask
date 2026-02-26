use crate::tui::app::LogEntry;
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::collections::VecDeque;

/// Renders the bottom-right log panel with timestamped success/error coloring.
pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, logs: &VecDeque<LogEntry>) {
    let lines = logs
        .iter()
        .map(|entry| {
            let msg_color = if entry.is_error {
                Color::Red
            } else {
                Color::Green
            };
            Line::from(vec![
                Span::styled(
                    format!("{} ", entry.time),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(entry.message.clone(), Style::default().fg(msg_color)),
            ])
        })
        .collect::<Vec<_>>();

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Log"))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
