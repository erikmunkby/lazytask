use super::layout::centered_rect;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub(super) fn render_keybindings_overlay(frame: &mut Frame, main_area: Rect) {
    let area = centered_rect(46, 66, main_area);
    frame.render_widget(Clear, area);

    let content = keybinding_lines().join("\n");
    let overlay = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Keybindings")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(overlay, area);
}

fn keybinding_lines() -> Vec<&'static str> {
    vec![
        "Nav: \u{2191}/\u{2193}",
        "Create: c",
        "Edit: e",
        "Start: s",
        "Done: x",
        "Open: o",
        "Delete: d",
        "Undo delete: u",
        "Quit: q (or Esc)",
        "",
        "Close: ? (or Esc)",
    ]
}

#[cfg(test)]
mod tests {
    use super::keybinding_lines;

    #[test]
    fn includes_delete_and_undo_rows() {
        let lines = keybinding_lines().join("\n");
        assert!(lines.contains("Delete: d"));
        assert!(lines.contains("Undo delete: u"));
    }

    #[test]
    fn includes_explicit_close_hint() {
        let lines = keybinding_lines().join("\n");
        assert!(lines.contains("Close: ? (or Esc)"));
    }
}
