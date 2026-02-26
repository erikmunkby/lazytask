use crate::domain::{Task, TaskType};
use crate::tui::actions::CreateField;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub time: String,
    pub message: String,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
pub enum EditorMode {
    Create,
    Edit { file_name: String },
}

#[derive(Debug, Clone)]
pub struct CreateState {
    pub editor_mode: EditorMode,
    pub active_field: CreateField,
    pub title: String,
    pub task_type: TaskType,
    pub details: String,
    pub cursor_pos: usize,
}

impl CreateState {
    /// Initializes empty state for creating a brand-new task.
    pub(super) fn new_create() -> Self {
        Self {
            editor_mode: EditorMode::Create,
            active_field: CreateField::Title,
            title: String::new(),
            task_type: TaskType::Task,
            details: String::new(),
            cursor_pos: 0,
        }
    }

    /// Initializes edit state from an existing task snapshot.
    pub(super) fn from_task(task: &Task) -> Self {
        Self {
            editor_mode: EditorMode::Edit {
                file_name: task.file_name.clone(),
            },
            active_field: CreateField::Title,
            title: task.title.clone(),
            task_type: task.task_type,
            details: task.details.clone(),
            cursor_pos: task.title.len(),
        }
    }

    /// True when this state is editing an existing task rather than creating one.
    pub(crate) fn is_editing(&self) -> bool {
        matches!(self.editor_mode, EditorMode::Edit { .. })
    }

    /// Returns the text content for whichever field is currently active.
    pub(super) fn active_text(&self) -> &str {
        match self.active_field {
            CreateField::Title => &self.title,
            CreateField::Type => self.task_type.as_str(),
            CreateField::Details => &self.details,
        }
    }

    /// Inserts a character or toggles task type when the type field is active.
    pub(super) fn insert_char(&mut self, ch: char) {
        match self.active_field {
            CreateField::Title => {
                self.title.insert(self.cursor_pos, ch);
                self.cursor_pos += ch.len_utf8();
            }
            CreateField::Type => {
                self.task_type = match self.task_type {
                    TaskType::Task => TaskType::Bug,
                    TaskType::Bug => TaskType::Task,
                };
            }
            CreateField::Details => {
                self.details.insert(self.cursor_pos, ch);
                self.cursor_pos += ch.len_utf8();
            }
        }
    }

    /// Deletes one previous Unicode scalar from the active text field.
    pub(super) fn delete_char(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        match self.active_field {
            CreateField::Title => {
                let prev = prev_char_boundary(&self.title, self.cursor_pos);
                self.title.drain(prev..self.cursor_pos);
                self.cursor_pos = prev;
            }
            CreateField::Type => {}
            CreateField::Details => {
                let prev = prev_char_boundary(&self.details, self.cursor_pos);
                self.details.drain(prev..self.cursor_pos);
                self.cursor_pos = prev;
            }
        }
    }

    /// Moves cursor left to the previous UTF-8 character boundary.
    pub(super) fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            let text = self.active_text();
            self.cursor_pos = prev_char_boundary(text, self.cursor_pos);
        }
    }

    /// Moves cursor right to the next UTF-8 character boundary.
    pub(super) fn move_cursor_right(&mut self) {
        let len = self.active_text().len();
        if self.cursor_pos < len {
            let text = match self.active_field {
                CreateField::Title => &self.title,
                CreateField::Type => return,
                CreateField::Details => &self.details,
            };
            self.cursor_pos = next_char_boundary(text, self.cursor_pos);
        }
    }

    /// Changes active field and snaps cursor to the end of that field.
    pub(super) fn switch_to(&mut self, field: CreateField) {
        self.active_field = field;
        self.cursor_pos = self.active_text().len();
    }
}

/// Returns the nearest valid previous UTF-8 character boundary.
fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos.saturating_sub(1);
    while p > 0 && !s.is_char_boundary(p) {
        p -= 1;
    }
    p
}

/// Returns the nearest valid next UTF-8 character boundary.
fn next_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos + 1;
    while p < s.len() && !s.is_char_boundary(p) {
        p += 1;
    }
    p
}

#[derive(Debug, Clone)]
pub enum Mode {
    Normal,
    Creating(CreateState),
    Keybindings,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub preview_text: String,
    pub log_entries: VecDeque<LogEntry>,
    pub last_deleted: Option<Task>,
    pub mode: Mode,
    pub should_quit: bool,
}
