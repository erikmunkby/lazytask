use super::layout::centered_rect;
use crate::domain::TITLE_CHAR_LIMIT;
use crate::tui::actions::CreateField;
use crate::tui::app::CreateState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub(super) fn render_create_modal(frame: &mut Frame, state: &CreateState, main_area: Rect) {
    let area = centered_rect(70, 40, main_area);
    frame.render_widget(Clear, area);

    let label_style = Style::default().fg(Color::Magenta);
    let active_style = Style::default().fg(Color::Cyan);
    let hint_style = Style::default().fg(Color::DarkGray);

    let title_style = if state.active_field == CreateField::Title {
        active_style
    } else {
        Style::default()
    };
    let type_style = if state.active_field == CreateField::Type {
        active_style
    } else {
        Style::default()
    };
    let desc_style = if state.active_field == CreateField::Details {
        active_style
    } else {
        Style::default()
    };

    let type_hint = if state.active_field == CreateField::Type {
        " (any key to toggle)"
    } else {
        ""
    };

    let cursor = "█";

    let title_display = if state.active_field == CreateField::Title {
        format!("{}{}", state.title, cursor)
    } else {
        state.title.clone()
    };

    let desc_display = if state.active_field == CreateField::Details {
        format!("{}{}", state.details, cursor)
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
        Span::styled(
            if title_display.is_empty() {
                format!("<title>{cursor}")
            } else {
                title_display
            },
            title_style,
        ),
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

    if desc_display.is_empty() && state.active_field == CreateField::Details {
        lines.push(Line::from(vec![Span::styled(
            format!("  <details>{cursor}"),
            desc_style,
        )]));
    } else if desc_display.is_empty() {
        lines.push(Line::from(vec![Span::styled("  <details>", desc_style)]));
    } else {
        for desc_line in desc_display.lines() {
            lines.push(Line::from(vec![Span::styled(
                format!("  {desc_line}"),
                desc_style,
            )]));
        }
        if desc_display.ends_with('\n') {
            lines.push(Line::from(vec![Span::styled(
                format!("  {cursor}"),
                desc_style,
            )]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Tab/↑↓: navigate | Enter: next | C-s: submit | Esc: cancel",
        hint_style,
    )]));

    let modal = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Create Task")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(modal, area);
}
