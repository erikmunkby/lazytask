use crate::config::LimitsConfig;
use crate::domain::{Task, TaskStatus, TaskType, format_relative};
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

const DIMMED: Style = Style::new().fg(Color::DarkGray);
const COLUMN_COUNT: usize = 5;
const COLUMN_SPACING: u16 = 1;
const SPACER_COL_WIDTH: u16 = 2;
const STATUS_COL_WIDTH: u16 = 6;
const TYPE_COL_WIDTH: u16 = 4;
const UPDATED_COL_WIDTH: u16 = 8;

/// Maps task status to compact table display tokens.
fn status_display(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "todo",
        TaskStatus::InProgress => "->",
        TaskStatus::Done => "done",
        TaskStatus::Discard => "X",
    }
}

/// Renders the left-side task table with status grouping separator.
pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    tasks: &[Task],
    selected_index: usize,
    limits: LimitsConfig,
) {
    let now = chrono::Utc::now();
    let separator_index = completed_separator_index(tasks);
    let mut rows = Vec::with_capacity(tasks.len() + usize::from(separator_index.is_some()));
    for (index, task) in tasks.iter().enumerate() {
        if separator_index == Some(index) {
            rows.push(separator_row());
        }
        rows.push(task_row(task, now));
    }

    let table = Table::new(rows, column_constraints())
        .column_spacing(COLUMN_SPACING)
        .header(
            Row::new(vec!["Title", "", "Status", "Type", "Updated"])
                .style(Style::default().add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(wip_title(tasks, limits))
                .border_style(Style::default().fg(Color::Green)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    let mut table_state = TableState::default().with_selected(Some(display_selected_index(
        selected_index,
        separator_index,
    )));
    frame.render_stateful_widget(table, area, &mut table_state);
}

/// Builds one styled table row for a task item.
fn task_row<'a>(task: &'a Task, now: chrono::DateTime<chrono::Utc>) -> Row<'a> {
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
    ])
}

/// Returns a dimmed divider row used between active and completed groups.
fn separator_row() -> Row<'static> {
    let dash = "----".repeat(50);
    let cells = (0..COLUMN_COUNT)
        .map(|_| Cell::from(dash.clone()))
        .collect::<Vec<_>>();
    Row::new(cells).style(DIMMED)
}

/// Defines fixed table column constraints shared across renders.
fn column_constraints() -> [Constraint; COLUMN_COUNT] {
    [
        Constraint::Fill(1),
        // Explicit spacer only between Title and Status.
        Constraint::Length(SPACER_COL_WIDTH),
        Constraint::Length(STATUS_COL_WIDTH),
        Constraint::Length(TYPE_COL_WIDTH),
        Constraint::Length(UPDATED_COL_WIDTH),
    ]
}

/// Returns insertion index for the active/completed group separator row.
fn completed_separator_index(tasks: &[Task]) -> Option<usize> {
    let has_active = tasks
        .iter()
        .any(|task| matches!(task.status, TaskStatus::InProgress | TaskStatus::Todo));
    let first_completed = tasks
        .iter()
        .position(|task| matches!(task.status, TaskStatus::Done | TaskStatus::Discard));
    if has_active { first_completed } else { None }
}

/// Builds the block title line with colored WIP limit indicators.
fn wip_title(tasks: &[Task], limits: LimitsConfig) -> Line<'static> {
    let todo_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Todo)
        .count();
    let ip_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::InProgress)
        .count();

    Line::from(vec![
        Span::raw("Tasks "),
        Span::styled(
            format!("[todo {todo_count}/{}]", limits.todo),
            Style::new().fg(limit_color(todo_count, limits.todo)),
        ),
        Span::raw(" "),
        Span::styled(
            format!("[wip {ip_count}/{}]", limits.in_progress),
            Style::new().fg(limit_color(ip_count, limits.in_progress)),
        ),
    ])
}

/// Returns a color reflecting how close `count` is to `limit`.
///
/// - At limit (100%) → Red
/// - 75%+ of limit → Yellow
/// - Below 75% → DarkGray (dimmed)
fn limit_color(count: usize, limit: usize) -> Color {
    if count >= limit {
        Color::Red
    } else if count * 100 >= limit * 75 {
        Color::Yellow
    } else {
        Color::DarkGray
    }
}

/// Translates task index to rendered row index when a separator row is present.
fn display_selected_index(selected_index: usize, separator_index: Option<usize>) -> usize {
    match separator_index {
        Some(index) if selected_index >= index => selected_index + 1,
        _ => selected_index,
    }
}

#[cfg(test)]
mod tests {
    use super::{completed_separator_index, display_selected_index, limit_color, status_display};
    use crate::domain::{Task, TaskStatus, TaskType};
    use chrono::{TimeZone, Utc};
    use ratatui::style::Color;

    #[test]
    fn status_display_is_compact_for_in_progress_only() {
        assert_eq!(status_display(TaskStatus::Todo), "todo");
        assert_eq!(status_display(TaskStatus::InProgress), "->");
        assert_eq!(status_display(TaskStatus::Done), "done");
        assert_eq!(status_display(TaskStatus::Discard), "X");
    }

    #[test]
    fn separator_index_matches_first_completed_group() {
        let tasks = vec![
            task("a", TaskStatus::InProgress),
            task("b", TaskStatus::Todo),
            task("c", TaskStatus::Done),
            task("d", TaskStatus::Discard),
        ];
        assert_eq!(completed_separator_index(&tasks), Some(2));
    }

    #[test]
    fn separator_index_none_when_only_one_group_present() {
        let active_only = vec![
            task("a", TaskStatus::InProgress),
            task("b", TaskStatus::Todo),
        ];
        let completed_only = vec![task("c", TaskStatus::Done), task("d", TaskStatus::Discard)];
        assert_eq!(completed_separator_index(&active_only), None);
        assert_eq!(completed_separator_index(&completed_only), None);
    }

    #[test]
    fn selected_display_index_skips_separator_row() {
        assert_eq!(display_selected_index(0, Some(2)), 0);
        assert_eq!(display_selected_index(1, Some(2)), 1);
        assert_eq!(display_selected_index(2, Some(2)), 3);
        assert_eq!(display_selected_index(3, Some(2)), 4);
        assert_eq!(display_selected_index(1, None), 1);
    }

    #[test]
    fn limit_color_dimmed_when_well_under() {
        assert_eq!(limit_color(0, 20), Color::DarkGray);
        assert_eq!(limit_color(14, 20), Color::DarkGray);
    }

    #[test]
    fn limit_color_yellow_when_approaching() {
        assert_eq!(limit_color(15, 20), Color::Yellow); // exactly 75%
        assert_eq!(limit_color(19, 20), Color::Yellow);
        assert_eq!(limit_color(3, 4), Color::Yellow); // 75%
    }

    #[test]
    fn limit_color_dimmed_for_small_limits_under_threshold() {
        // 2/3 = 66.7% → still under 75%
        assert_eq!(limit_color(2, 3), Color::DarkGray);
    }

    #[test]
    fn limit_color_red_at_limit() {
        assert_eq!(limit_color(20, 20), Color::Red);
        assert_eq!(limit_color(3, 3), Color::Red);
        assert_eq!(limit_color(1, 1), Color::Red);
    }

    fn task(title: &str, status: TaskStatus) -> Task {
        let now = Utc.timestamp_opt(0, 0).single().unwrap();
        Task {
            title: title.to_string(),
            file_name: format!("{title}.md"),
            status,
            task_type: TaskType::Task,
            discard_note: None,
            details: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}
