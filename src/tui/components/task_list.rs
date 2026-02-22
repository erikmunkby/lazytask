use crate::domain::{Task, TaskStatus, TaskType, format_relative};
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

const DIMMED: Style = Style::new().fg(Color::DarkGray);

fn status_display(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "todo",
        TaskStatus::InProgress => "->",
        TaskStatus::Done => "done",
        TaskStatus::Discard => "X",
    }
}

pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    tasks: &[Task],
    selected_index: usize,
) {
    let now = chrono::Utc::now();
    let rows = tasks
        .iter()
        .map(|task| {
            let status_style = match task.status {
                TaskStatus::Todo => DIMMED,
                TaskStatus::InProgress => Style::new().fg(Color::Magenta),
                TaskStatus::Done => Style::new().fg(Color::Green),
                TaskStatus::Discard => Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
            };
            let type_style = match task.task_type {
                TaskType::Task => Style::new().fg(Color::Blue),
                TaskType::Bug => Style::new().fg(Color::Red),
            };
            Row::new(vec![
                Cell::from(task.title.as_str()),
                Cell::from(""),
                Cell::from(status_display(task.status)).style(status_style),
                Cell::from(task.task_type.as_str()).style(type_style),
                Cell::from(format_relative(task.updated_at, now)).style(DIMMED),
                Cell::from(format_relative(task.created_at, now)).style(DIMMED),
            ])
        })
        .collect::<Vec<_>>();

    let table = Table::new(
        rows,
        [
            Constraint::Fill(1),
            // Explicit spacer only between Title and Status.
            Constraint::Length(2),
            Constraint::Length(6),
            Constraint::Length(4),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .column_spacing(1)
    .header(
        Row::new(vec!["Title", "", "Status", "Type", "Updated", "Created"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Tasks")
            .border_style(Style::default().fg(Color::Green)),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    let mut table_state = TableState::default().with_selected(Some(selected_index));
    frame.render_stateful_widget(table, area, &mut table_state);
}

#[cfg(test)]
mod tests {
    use super::status_display;
    use crate::domain::TaskStatus;

    #[test]
    fn status_display_is_compact_for_in_progress_only() {
        assert_eq!(status_display(TaskStatus::Todo), "todo");
        assert_eq!(status_display(TaskStatus::InProgress), "->");
        assert_eq!(status_display(TaskStatus::Done), "done");
        assert_eq!(status_display(TaskStatus::Discard), "X");
    }
}
