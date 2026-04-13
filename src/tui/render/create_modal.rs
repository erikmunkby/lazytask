use super::layout::centered_rect;
use crate::domain::{TITLE_CHAR_LIMIT, TaskType};
use crate::tui::actions::CreateField;
use crate::tui::app::CreateState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

const MODAL_WIDTH_PERCENT: u16 = 82;
const MODAL_HEIGHT_PERCENT: u16 = 68;

/// Renders the create/edit modal with field highlighting and inline validation hints.
pub(super) fn render_create_modal(frame: &mut Frame, state: &CreateState, main_area: Rect) {
    let area = centered_rect(MODAL_WIDTH_PERCENT, MODAL_HEIGHT_PERCENT, main_area);
    frame.render_widget(Clear, area);

    let label_style = Style::default().fg(Color::Magenta);
    let active_style = Style::default().fg(Color::Cyan);
    let hint_style = Style::default().fg(Color::DarkGray);

    let title_is_active = state.active_field == CreateField::Title;
    let title_style = value_style(state.title.is_empty(), title_is_active, active_style);
    let details_is_active = state.active_field == CreateField::Details;
    let type_style = task_type_style(&state.task_type, state.active_field == CreateField::Type);
    let desc_style = value_style(state.details.is_empty(), details_is_active, active_style);

    let type_hint = if state.active_field == CreateField::Type {
        " (any key to toggle)"
    } else {
        ""
    };

    let cursor = "█";

    let title_display = title_value_text(&state.title, title_is_active, state.cursor_pos, cursor);

    let desc_display = if details_is_active {
        with_cursor(&state.details, state.cursor_pos, cursor)
    } else {
        state.details.clone()
    };

    let title_len = state.title.trim().len();
    let over_limit = title_len > TITLE_CHAR_LIMIT;
    let counter_style = if over_limit {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Green)
    };

    let mut title_line = vec![
        Span::styled("# ", label_style),
        Span::styled(title_display, title_style),
    ];

    if state.active_field == CreateField::Title {
        title_line.push(Span::styled(
            format!(" ({}/{})", title_len, TITLE_CHAR_LIMIT),
            counter_style,
        ));
    }

    let mut lines = vec![Line::from(title_line)];

    if over_limit {
        lines.push(Line::from(vec![Span::styled(
            "Title too long!",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("type: ", label_style),
        Span::styled(state.task_type.as_str(), type_style),
        Span::styled(type_hint, hint_style),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("details:", label_style)]));

    if state.details.is_empty() && state.active_field == CreateField::Details {
        lines.push(Line::from(vec![Span::styled(
            format!("  <details>{cursor}"),
            desc_style,
        )]));
    } else if state.details.is_empty() {
        lines.push(Line::from(vec![Span::styled("  <details>", desc_style)]));
    } else {
        for desc_line in desc_display.split('\n') {
            lines.push(Line::from(vec![Span::styled(
                format!("  {desc_line}"),
                desc_style,
            )]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Tab/↑↓: navigate | Enter: next | C-v: paste | C-u: clear line | C-s: save | Esc: cancel",
        hint_style,
    )]));

    let modal_title = if state.is_editing() {
        "Edit Task"
    } else {
        "Create Task"
    };

    let modal = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(modal_title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(modal, area);
}

/// Inserts a visual cursor marker at a UTF-8-safe byte position.
fn with_cursor(text: &str, cursor_pos: usize, cursor: &str) -> String {
    let mut pos = cursor_pos.min(text.len());
    while pos > 0 && !text.is_char_boundary(pos) {
        pos -= 1;
    }

    let (left, right) = text.split_at(pos);
    format!("{left}{cursor}{right}")
}

/// Builds title display text with placeholder and cursor behavior.
fn title_value_text(title: &str, is_active: bool, cursor_pos: usize, cursor: &str) -> String {
    if title.is_empty() {
        if is_active {
            format!("<title>{cursor}")
        } else {
            "<title>".to_string()
        }
    } else if is_active {
        with_cursor(title, cursor_pos, cursor)
    } else {
        title.to_string()
    }
}

/// Chooses value styling for empty vs active field states.
fn value_style(is_empty: bool, is_active: bool, active_style: Style) -> Style {
    if is_empty {
        Style::default().fg(Color::DarkGray)
    } else if is_active {
        active_style
    } else {
        Style::default()
    }
}

/// Returns task-type color styling, bolding when the field is active.
fn task_type_style(task_type: &TaskType, is_active: bool) -> Style {
    let base = match task_type {
        TaskType::Task => Style::default().fg(Color::Blue),
        TaskType::Bug => Style::default().fg(Color::Red),
    };

    if is_active {
        base.add_modifier(Modifier::BOLD)
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::{task_type_style, title_value_text, value_style, with_cursor};
    use crate::domain::TaskType;
    use ratatui::style::{Color, Modifier};

    #[test]
    fn places_cursor_at_requested_position() {
        assert_eq!(with_cursor("abcd", 2, "|"), "ab|cd");
    }

    #[test]
    fn clamps_cursor_to_end_for_out_of_bounds_positions() {
        assert_eq!(with_cursor("abcd", 99, "|"), "abcd|");
    }

    #[test]
    fn handles_cursor_at_start() {
        assert_eq!(with_cursor("abcd", 0, "|"), "|abcd");
    }

    #[test]
    fn moves_back_to_char_boundary() {
        let text = "aé";
        assert_eq!(with_cursor(text, 2, "|"), "a|é");
    }

    #[test]
    fn empty_title_placeholder_hides_cursor_when_inactive() {
        assert_eq!(title_value_text("", false, 0, "|"), "<title>");
    }

    #[test]
    fn empty_title_placeholder_shows_cursor_when_active() {
        assert_eq!(title_value_text("", true, 0, "|"), "<title>|");
    }

    #[test]
    fn empty_title_placeholder_is_dimmed() {
        assert_eq!(
            value_style(
                true,
                false,
                ratatui::style::Style::default().fg(Color::Cyan)
            )
            .fg,
            Some(Color::DarkGray)
        );
    }

    #[test]
    fn empty_details_placeholder_is_dimmed_even_when_active() {
        assert_eq!(
            value_style(true, true, ratatui::style::Style::default().fg(Color::Cyan)).fg,
            Some(Color::DarkGray)
        );
    }

    #[test]
    fn task_type_style_matches_table_colors() {
        assert_eq!(
            task_type_style(&TaskType::Task, false).fg,
            Some(Color::Blue)
        );
        assert_eq!(task_type_style(&TaskType::Bug, false).fg, Some(Color::Red));
    }

    #[test]
    fn active_task_type_style_is_bold() {
        assert!(
            task_type_style(&TaskType::Task, true)
                .add_modifier
                .contains(Modifier::BOLD)
        );
    }
}
