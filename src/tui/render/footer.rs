use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

/// Renders the single-line footer hint strip.
pub(super) fn render_key_hints(frame: &mut Frame, area: Rect) {
    let key_style = Style::default().fg(Color::Cyan);
    let keys = key_hints();
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

/// Returns the compact set of primary key hints shown in the footer.
fn key_hints() -> [&'static str; 7] {
    [
        "Nav: ↑/↓",
        "Create: c",
        "Edit: e",
        "Open: o",
        "Quit: q",
        "Delete: d (Undo: u)",
        "Keybindings: ?",
    ]
}

#[cfg(test)]
mod tests {
    use super::key_hints;

    #[test]
    fn footer_keeps_only_primary_hints() {
        assert_eq!(
            key_hints(),
            [
                "Nav: ↑/↓",
                "Create: c",
                "Edit: e",
                "Open: o",
                "Quit: q",
                "Delete: d (Undo: u)",
                "Keybindings: ?",
            ]
        );
    }
}
