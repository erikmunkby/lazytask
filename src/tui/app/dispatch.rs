use super::{App, CreateState, LOG_CAPACITY, LogEntry, Mode};
use crate::domain::{Task, TaskStatus};
use crate::services::CreateTaskInput;
use crate::tui::actions::Action;
use chrono::Local;

impl App {
    /// Applies one UI action and mutates state plus persisted tasks as needed.
    pub fn dispatch(&mut self, action: Action) {
        match action {
            Action::RefreshTasks => match self.service.list_tasks(None, None) {
                Ok(mut tasks) => {
                    sort_tasks_for_tui(&mut tasks);
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
                self.state.mode = Mode::Creating(CreateState::new_create());
            }
            Action::EditSelectedRequested => {
                if let Some(task) = self.selected_task().cloned() {
                    if task.status == TaskStatus::Discard {
                        self.push_log(
                            "discarded tasks are terminal; delete instead".to_string(),
                            true,
                        );
                        return;
                    }
                    self.state.mode = Mode::Creating(CreateState::from_task(&task));
                }
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
            Action::EditTaskSubmitted {
                file_name,
                title,
                task_type,
                details,
            } => match self
                .service
                .edit_task(&file_name, title, task_type, details)
            {
                Ok(task) => {
                    self.push_log(format!("task \"{}\" updated", task.title), false);
                    self.state.mode = Mode::Normal;
                    self.dispatch(Action::RefreshTasks);
                }
                Err(err) => {
                    self.push_log(format!("{err}"), true);
                }
            },
            Action::DeleteSelected => {
                if let Some(task) = self.selected_task().cloned() {
                    match self.service.delete_task_exact(&task) {
                        Ok(_) => {
                            if task.status == TaskStatus::Discard {
                                self.state.last_deleted = None;
                                self.dispatch(Action::TaskOperationSucceeded {
                                    message: format!("discarded task \"{}\" deleted", task.title),
                                });
                            } else {
                                self.state.last_deleted = Some(task.clone());
                                self.dispatch(Action::TaskOperationSucceeded {
                                    message: format!(
                                        "task \"{}\" deleted (press u to undo)",
                                        task.title
                                    ),
                                });
                            }
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
                    if task.status == TaskStatus::Discard {
                        self.dispatch(Action::TaskOperationFailed {
                            message: "start failed: discarded tasks are terminal".to_string(),
                        });
                        return;
                    }
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
                    if task.status == TaskStatus::Discard {
                        self.dispatch(Action::TaskOperationFailed {
                            message: "done failed: discarded tasks are terminal".to_string(),
                        });
                        return;
                    }
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
            Action::OpenSelectedInEditor => {
                if let Some(task) = self.selected_task().cloned() {
                    match self.service.open_task_in_editor(&task) {
                        Ok(editor) => self.dispatch(Action::TaskOperationSucceeded {
                            message: format!("opened \"{}\" in {editor}", task.title),
                        }),
                        Err(err) => self.dispatch(Action::TaskOperationFailed {
                            message: format!("open failed: {err}"),
                        }),
                    }
                }
            }
            Action::TaskOperationSucceeded { message } => self.push_log(message, false),
            Action::TaskOperationFailed { message } => self.push_log(message, true),
            Action::Quit => self.state.should_quit = true,
        }
    }

    /// Returns the currently highlighted task, if any.
    fn selected_task(&self) -> Option<&Task> {
        self.state.tasks.get(self.state.selected_index)
    }

    /// Refreshes the right-hand preview with raw markdown for the selected task.
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

    /// Appends a log entry while enforcing fixed panel capacity.
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

/// Sorts tasks for TUI grouping: in-progress, todo, done, then discard.
fn sort_tasks_for_tui(tasks: &mut [Task]) {
    tasks.sort_by(|a, b| {
        status_group_rank(a.status)
            .cmp(&status_group_rank(b.status))
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });
}

/// Provides status group ordering rank used by TUI sorting.
fn status_group_rank(status: TaskStatus) -> u8 {
    match status {
        TaskStatus::InProgress => 0,
        TaskStatus::Todo => 1,
        TaskStatus::Done => 2,
        TaskStatus::Discard => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::sort_tasks_for_tui;
    use crate::domain::{Task, TaskStatus, TaskType};
    use chrono::{TimeZone, Utc};

    #[test]
    fn sort_tasks_groups_by_status_then_updated_desc() {
        let mut tasks = vec![
            task("done-new", TaskStatus::Done, 5),
            task("todo-old", TaskStatus::Todo, 3),
            task("in-progress-old", TaskStatus::InProgress, 1),
            task("discard-new", TaskStatus::Discard, 6),
            task("todo-new", TaskStatus::Todo, 4),
            task("in-progress-new", TaskStatus::InProgress, 2),
        ];

        sort_tasks_for_tui(&mut tasks);

        let titles = tasks
            .iter()
            .map(|task| task.title.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            titles,
            vec![
                "in-progress-new",
                "in-progress-old",
                "todo-new",
                "todo-old",
                "done-new",
                "discard-new"
            ]
        );
    }

    fn task(title: &str, status: TaskStatus, updated_at: i64) -> Task {
        let timestamp = Utc.timestamp_opt(updated_at, 0).single().unwrap();
        Task {
            title: title.to_string(),
            file_name: format!("{title}.md"),
            status,
            task_type: TaskType::Task,
            discard_note: None,
            details: String::new(),
            created_at: timestamp,
            updated_at: timestamp,
        }
    }
}
