use super::{App, CreateState, LOG_CAPACITY, LogEntry, Mode};
use crate::domain::{Task, TaskType};
use crate::services::CreateTaskInput;
use crate::tui::actions::{Action, CreateField};
use chrono::Local;

impl App {
    pub fn dispatch(&mut self, action: Action) {
        match action {
            Action::RefreshTasks => match self.service.list_tasks(None, None) {
                Ok(tasks) => {
                    self.state.tasks = tasks;
                    if self.state.selected_index >= self.state.tasks.len() {
                        self.state.selected_index = self.state.tasks.len().saturating_sub(1);
                    }
                    self.refresh_preview();
                }
                Err(err) => self.push_log(format!("error refreshing tasks: {err}"), true),
            },
            Action::CheckLearningHint => match self.service.learnings_line_count() {
                Ok(count) if count > self.learn_threshold => self.push_log(
                    format!("LEARNINGS.md has {count} lines. Ask your AI agent to run `lt learn`"),
                    false,
                ),
                Ok(_) => {}
                Err(err) => self.push_log(format!("error reading learnings count: {err}"), true),
            },
            Action::MoveSelectionUp => {
                self.state.selected_index = self.state.selected_index.saturating_sub(1);
                self.refresh_preview();
            }
            Action::MoveSelectionDown => {
                if !self.state.tasks.is_empty() {
                    self.state.selected_index = (self.state.selected_index + 1)
                        .min(self.state.tasks.len().saturating_sub(1));
                }
                self.refresh_preview();
            }
            Action::CreateTaskRequested => {
                self.state.mode = Mode::Creating(CreateState {
                    active_field: CreateField::Title,
                    title: String::new(),
                    task_type: TaskType::Task,
                    details: String::new(),
                    cursor_pos: 0,
                });
            }
            Action::CreateTaskSubmitted {
                title,
                task_type,
                details,
            } => {
                let result = self.service.create_task(CreateTaskInput {
                    title: title.clone(),
                    task_type,
                    details,
                    start: false,
                    require_details: false,
                });
                match result {
                    Ok(task) => {
                        self.push_log(format!("task \"{}\" created", task.title), false);
                        self.state.mode = Mode::Normal;
                        self.dispatch(Action::RefreshTasks);
                    }
                    Err(err) => {
                        self.push_log(format!("{err}"), true);
                    }
                }
            }
            Action::DeleteSelected => {
                if let Some(task) = self.selected_task().cloned() {
                    match self.service.delete_task(&task.file_name) {
                        Ok(_) => {
                            self.state.last_deleted = Some(task.clone());
                            self.dispatch(Action::TaskOperationSucceeded {
                                message: format!(
                                    "task \"{}\" deleted (press u to undo)",
                                    task.title
                                ),
                            });
                            self.dispatch(Action::RefreshTasks);
                        }
                        Err(err) => self.dispatch(Action::TaskOperationFailed {
                            message: format!("delete failed: {err}"),
                        }),
                    }
                }
            }
            Action::UndoDelete => {
                if let Some(task) = self.state.last_deleted.clone() {
                    match self.service.restore_task(&task) {
                        Ok(restored) => {
                            self.state.last_deleted = None;
                            self.dispatch(Action::TaskOperationSucceeded {
                                message: format!("task \"{}\" restored", restored.title),
                            });
                            self.dispatch(Action::RefreshTasks);
                        }
                        Err(err) => self.dispatch(Action::TaskOperationFailed {
                            message: format!("undo failed: {err}"),
                        }),
                    }
                }
            }
            Action::StartSelected => {
                if let Some(task) = self.selected_task().cloned() {
                    match self.service.start_task(&task.file_name) {
                        Ok(updated) => {
                            self.dispatch(Action::TaskOperationSucceeded {
                                message: format!("task \"{}\" moved to in-progress", updated.title),
                            });
                            self.dispatch(Action::RefreshTasks);
                        }
                        Err(err) => self.dispatch(Action::TaskOperationFailed {
                            message: format!("start failed: {err}"),
                        }),
                    }
                }
            }
            Action::DoneSelected => {
                if let Some(task) = self.selected_task().cloned() {
                    match self.service.done_task_without_learning(&task.file_name) {
                        Ok(updated) => {
                            self.dispatch(Action::TaskOperationSucceeded {
                                message: format!("task \"{}\" moved to done", updated.title),
                            });
                            self.dispatch(Action::RefreshTasks);
                        }
                        Err(err) => self.dispatch(Action::TaskOperationFailed {
                            message: format!("done failed: {err}"),
                        }),
                    }
                }
            }
            Action::TaskOperationSucceeded { message } => self.push_log(message, false),
            Action::TaskOperationFailed { message } => self.push_log(message, true),
            Action::Quit => self.state.should_quit = true,
        }
    }

    fn selected_task(&self) -> Option<&Task> {
        self.state.tasks.get(self.state.selected_index)
    }

    fn refresh_preview(&mut self) {
        if let Some(task) = self.selected_task() {
            self.state.preview_text = match self.service.read_task_content(task) {
                Ok(content) => content,
                Err(err) => format!("Unable to read task preview: {err}"),
            };
        } else {
            self.state.preview_text = "No tasks".to_string();
        }
    }

    pub(super) fn push_log(&mut self, message: String, is_error: bool) {
        self.state.log_entries.push_back(LogEntry {
            time: Local::now().format("%H:%M:%S").to_string(),
            message,
            is_error,
        });

        while self.state.log_entries.len() > LOG_CAPACITY {
            self.state.log_entries.pop_front();
        }
    }
}
