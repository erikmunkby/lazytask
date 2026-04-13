use crate::config::LimitsConfig;
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

    /// Inserts a string at the current cursor position in the active text field.
    pub(super) fn insert_str(&mut self, s: &str) {
        match self.active_field {
            CreateField::Title => {
                self.title.insert_str(self.cursor_pos, s);
                self.cursor_pos += s.len();
            }
            CreateField::Type => {}
            CreateField::Details => {
                self.details.insert_str(self.cursor_pos, s);
                self.cursor_pos += s.len();
            }
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

    /// Deletes the word before the cursor (Alt+Backspace behaviour).
    pub(super) fn delete_word_backward(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        match self.active_field {
            CreateField::Title => {
                let boundary = prev_word_boundary(&self.title, self.cursor_pos);
                self.title.drain(boundary..self.cursor_pos);
                self.cursor_pos = boundary;
            }
            CreateField::Type => {}
            CreateField::Details => {
                let boundary = prev_word_boundary(&self.details, self.cursor_pos);
                self.details.drain(boundary..self.cursor_pos);
                self.cursor_pos = boundary;
            }
        }
    }

    /// Deletes from the cursor back to the start of the current line (Ctrl+U).
    pub(super) fn delete_to_line_start(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        match self.active_field {
            CreateField::Title => {
                self.title.drain(0..self.cursor_pos);
                self.cursor_pos = 0;
            }
            CreateField::Type => {}
            CreateField::Details => {
                let line_start = self.details[..self.cursor_pos]
                    .rfind('\n')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                self.details.drain(line_start..self.cursor_pos);
                self.cursor_pos = line_start;
            }
        }
    }

    /// Moves the cursor left by one word.
    pub(super) fn move_cursor_word_left(&mut self) {
        if self.cursor_pos > 0 {
            let text = self.active_text();
            self.cursor_pos = prev_word_boundary(text, self.cursor_pos);
        }
    }

    /// Moves the cursor right by one word.
    pub(super) fn move_cursor_word_right(&mut self) {
        let len = self.active_text().len();
        if self.cursor_pos < len {
            let text = match self.active_field {
                CreateField::Title => &self.title,
                CreateField::Type => return,
                CreateField::Details => &self.details,
            };
            self.cursor_pos = next_word_boundary(text, self.cursor_pos);
        }
    }

    /// Moves the cursor to the start of the current line (or field start).
    pub(super) fn move_cursor_home(&mut self) {
        match self.active_field {
            CreateField::Title | CreateField::Type => self.cursor_pos = 0,
            CreateField::Details => {
                self.cursor_pos = self.details[..self.cursor_pos]
                    .rfind('\n')
                    .map(|i| i + 1)
                    .unwrap_or(0);
            }
        }
    }

    /// Moves the cursor to the end of the current line (or field end).
    pub(super) fn move_cursor_end(&mut self) {
        match self.active_field {
            CreateField::Title => self.cursor_pos = self.title.len(),
            CreateField::Type => {}
            CreateField::Details => {
                self.cursor_pos = self.details[self.cursor_pos..]
                    .find('\n')
                    .map(|i| self.cursor_pos + i)
                    .unwrap_or(self.details.len());
            }
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

/// Scans backward from `pos` to find the previous word boundary.
///
/// Skips trailing non-alphanumeric chars, then skips the alphanumeric word.
fn prev_word_boundary(s: &str, pos: usize) -> usize {
    let chars: Vec<(usize, char)> = s.char_indices().take_while(|(i, _)| *i < pos).collect();
    let mut idx = chars.len();
    // Skip non-alphanumeric characters
    while idx > 0 && !chars[idx - 1].1.is_alphanumeric() {
        idx -= 1;
    }
    // Skip alphanumeric word
    while idx > 0 && chars[idx - 1].1.is_alphanumeric() {
        idx -= 1;
    }
    if idx == 0 { 0 } else { chars[idx].0 }
}

/// Scans forward from `pos` to find the next word boundary.
///
/// Skips leading non-alphanumeric chars, then skips the alphanumeric word.
fn next_word_boundary(s: &str, pos: usize) -> usize {
    let mut chars = s.char_indices().skip_while(|(i, _)| *i < pos);
    // Skip non-alphanumeric characters
    let mut last_pos = pos;
    for (i, ch) in chars.by_ref() {
        if ch.is_alphanumeric() {
            last_pos = i;
            break;
        }
        last_pos = i + ch.len_utf8();
    }
    // Skip alphanumeric word
    for (i, ch) in s[last_pos..].char_indices() {
        if !ch.is_alphanumeric() {
            return last_pos + i;
        }
    }
    s.len()
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
    pub limits: LimitsConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn title_state(title: &str, cursor: usize) -> CreateState {
        CreateState {
            editor_mode: EditorMode::Create,
            active_field: CreateField::Title,
            title: title.to_string(),
            task_type: TaskType::Task,
            details: String::new(),
            cursor_pos: cursor,
        }
    }

    fn details_state(details: &str, cursor: usize) -> CreateState {
        CreateState {
            editor_mode: EditorMode::Create,
            active_field: CreateField::Details,
            title: String::new(),
            task_type: TaskType::Task,
            details: details.to_string(),
            cursor_pos: cursor,
        }
    }

    // -- prev_word_boundary / next_word_boundary --

    #[test]
    fn prev_word_boundary_from_end_of_word() {
        assert_eq!(prev_word_boundary("hello world", 11), 6);
    }

    #[test]
    fn prev_word_boundary_skips_spaces() {
        assert_eq!(prev_word_boundary("hello   world", 8), 0);
    }

    #[test]
    fn prev_word_boundary_at_start() {
        assert_eq!(prev_word_boundary("hello", 0), 0);
    }

    #[test]
    fn next_word_boundary_from_start() {
        assert_eq!(next_word_boundary("hello world", 0), 5);
    }

    #[test]
    fn next_word_boundary_skips_spaces() {
        assert_eq!(next_word_boundary("hello   world", 5), 13);
    }

    #[test]
    fn next_word_boundary_at_end() {
        assert_eq!(next_word_boundary("hello", 5), 5);
    }

    #[test]
    fn word_boundary_with_unicode() {
        let s = "héllo wörld";
        assert_eq!(prev_word_boundary(s, s.len()), 7);
        assert_eq!(next_word_boundary(s, 0), 6);
    }

    // -- delete_word_backward --

    #[test]
    fn delete_word_backward_removes_word() {
        let mut s = title_state("hello world", 11);
        s.delete_word_backward();
        assert_eq!(s.title, "hello ");
        assert_eq!(s.cursor_pos, 6);
    }

    #[test]
    fn delete_word_backward_at_start_is_noop() {
        let mut s = title_state("hello", 0);
        s.delete_word_backward();
        assert_eq!(s.title, "hello");
        assert_eq!(s.cursor_pos, 0);
    }

    // -- delete_to_line_start --

    #[test]
    fn delete_to_line_start_clears_title() {
        let mut s = title_state("hello world", 11);
        s.delete_to_line_start();
        assert_eq!(s.title, "");
        assert_eq!(s.cursor_pos, 0);
    }

    #[test]
    fn delete_to_line_start_in_multiline_details() {
        let mut s = details_state("line1\nline2 end", 15);
        s.delete_to_line_start();
        assert_eq!(s.details, "line1\n");
        assert_eq!(s.cursor_pos, 6);
    }

    // -- move_cursor_word_left / move_cursor_word_right --

    #[test]
    fn move_word_left_skips_word() {
        let mut s = title_state("hello world", 11);
        s.move_cursor_word_left();
        assert_eq!(s.cursor_pos, 6);
    }

    #[test]
    fn move_word_right_skips_word() {
        let mut s = title_state("hello world", 0);
        s.move_cursor_word_right();
        assert_eq!(s.cursor_pos, 5);
    }

    // -- move_cursor_home / move_cursor_end --

    #[test]
    fn home_moves_to_field_start() {
        let mut s = title_state("hello", 3);
        s.move_cursor_home();
        assert_eq!(s.cursor_pos, 0);
    }

    #[test]
    fn end_moves_to_field_end() {
        let mut s = title_state("hello", 2);
        s.move_cursor_end();
        assert_eq!(s.cursor_pos, 5);
    }

    #[test]
    fn home_in_multiline_goes_to_line_start() {
        let mut s = details_state("line1\nline2", 9);
        s.move_cursor_home();
        assert_eq!(s.cursor_pos, 6);
    }

    #[test]
    fn end_in_multiline_goes_to_line_end() {
        let mut s = details_state("line1\nline2", 6);
        s.move_cursor_end();
        assert_eq!(s.cursor_pos, 11);
    }
}
