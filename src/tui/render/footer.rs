use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub(super) fn render_key_hints(frame: &mut Frame, area: Rect) {
    let key_style = Style::default().fg(Color::Cyan);
    let keys = [
        "Nav: ↑/↓",
        "Create: c",
        "Edit: e",
        "Start: s",
        "Done: x",
        "Delete: d",
        "Undo delete: u",
        "Quit: q",
    ];
    let spans: Vec<Span> = keys
        .iter()
        .enumerate()
        .flat_map(|(i, &key)| {
            let mut v = vec![Span::styled(key, key_style)];
            if i < keys.len() - 1 {
                v.push(Span::raw(" | "));
            }
            v
        })
        .collect();
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
