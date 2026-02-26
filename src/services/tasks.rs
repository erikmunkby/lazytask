use super::query::resolve_query;
use super::{CreateTaskInput, ServiceError, TaskService};
use crate::config;
use crate::domain::{
    DomainError, Task, TaskStatus, normalize_escaped_newlines, normalize_file_name,
    parse_learning_lines, validate_details, validate_discard_note, validate_title,
};
use chrono::Utc;

impl TaskService {
    /// Bootstraps workspace state needed before any task command can succeed.
    ///
    /// This ensures a default `lazytask.toml` exists, creates the `.tasks` layout,
    /// and appends agent guidance to the configured prompt file when missing.
    pub fn init(&self) -> Result<(), ServiceError> {
        self.init_with_upgrade(false)
    }

    /// Bootstraps workspace state and optionally rewrites generated defaults.
    pub fn init_with_upgrade(&self, upgrade: bool) -> Result<(), ServiceError> {
        config::ensure_default_file_with_upgrade(&self.config, upgrade)?;
        self.storage.ensure_layout()?;
        self.storage
            .ensure_agent_prompt_guidance_with_upgrade(upgrade)?;
        Ok(())
    }

    /// Lists tasks with optional status/type filtering.
    pub fn list_tasks(
        &self,
        status: Option<TaskStatus>,
        task_type: Option<crate::domain::TaskType>,
    ) -> Result<Vec<Task>, ServiceError> {
        Ok(self.storage.list_tasks(status, task_type)?)
    }

    /// Resolves each query against current tasks and returns matched tasks in input order.
    pub fn get_tasks(&self, queries: &[String]) -> Result<Vec<Task>, ServiceError> {
        self.storage.require_layout()?;
        let all = self.storage.list_tasks(None, None)?;
        let mut out = Vec::new();

        for query in queries {
            out.push(resolve_query(&all, query)?);
        }

        Ok(out)
    }

    /// Creates a new task after validation, normalization, and status-limit checks.
    ///
    /// `input.start` determines whether the task starts in `in-progress` or `todo`.
    /// Duplicate detection blocks titles that already exist in normal flow buckets
    /// (`todo`, `in-progress`, `done`) while allowing recreation of discarded tasks.
    pub fn create_task(&self, input: CreateTaskInput) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let normalized_details = normalize_escaped_newlines(&input.details);
        validate_title(&input.title).map_err(validation_error)?;
        validate_details(&normalized_details, input.require_details).map_err(validation_error)?;

        let status = if input.start {
            TaskStatus::InProgress
        } else {
            TaskStatus::Todo
        };

        self.enforce_status_limit(status)?;

        let file_name = normalize_file_name(&input.title).map_err(validation_error)?;
        let blocking_statuses = [TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done];
        if self
            .storage
            .find_task_by_exact_file_name_in_statuses(&file_name, &blocking_statuses)?
            .is_some()
        {
            return Err(ServiceError::TaskAlreadyExists(input.title));
        }

        let now = Utc::now();
        Ok(self.storage.create_task(
            &input.title,
            status,
            input.task_type,
            &normalized_details,
            now,
        )?)
    }

    /// Moves a task into `in-progress`, enforcing active WIP limits.
    ///
    /// If the task is already in progress this is a no-op.
    pub fn start_task(&self, query: &str) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        self.enforce_status_limit(TaskStatus::InProgress)?;
        let task = self.resolve_task(query)?;
        if task.status == TaskStatus::InProgress {
            return Ok(task);
        }

        Ok(self
            .storage
            .move_task(&task, TaskStatus::InProgress, Utc::now())?)
    }

    /// Marks a task done and records one or more learning lines in `learnings.md`.
    ///
    /// Learnings are validated before state changes so malformed input never moves
    /// the task. On success the task is moved first, then the learning entry is
    /// appended using the same completion timestamp.
    pub fn done_task_with_learning(
        &self,
        query: &str,
        learning: &str,
    ) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let learning_lines = parse_learning_lines(learning).map_err(validation_error)?;
        let task = self.resolve_task(query)?;

        let now = Utc::now();
        let moved = self.storage.move_task(&task, TaskStatus::Done, now)?;
        self.storage
            .append_learning(now, &moved.title, &learning_lines)?;

        Ok(moved)
    }

    /// Marks a task as done without recording learnings.
    pub fn done_task_without_learning(&self, query: &str) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let task = self.resolve_task(query)?;

        Ok(self
            .storage
            .move_task(&task, TaskStatus::Done, Utc::now())?)
    }

    /// Moves a task to the discard bucket without attaching a note.
    pub fn discard_task(&self, query: &str) -> Result<Task, ServiceError> {
        self.discard_task_internal(query, None)
    }

    /// Moves a task to discard and persists a validated discard note.
    pub fn discard_task_with_note(
        &self,
        query: &str,
        discard_note: &str,
    ) -> Result<Task, ServiceError> {
        let discard_note = validate_discard_note(discard_note).map_err(validation_error)?;
        self.discard_task_internal(query, Some(discard_note))
    }

    /// Shared discard implementation used by note/no-note command variants.
    fn discard_task_internal(
        &self,
        query: &str,
        discard_note: Option<String>,
    ) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let mut task = self.resolve_task(query)?;
        task.discard_note = discard_note;

        Ok(self
            .storage
            .move_task(&task, TaskStatus::Discard, Utc::now())?)
    }

    /// Deletes a resolved task file from its current bucket.
    pub fn delete_task(&self, query: &str) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let task = self.resolve_task(query)?;

        self.storage.delete_task(&task)?;
        Ok(task)
    }

    /// Deletes a task by exact identity without query resolution.
    pub fn delete_task_exact(&self, task: &Task) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        self.storage.delete_task(task)?;
        Ok(task.clone())
    }

    /// Restores a previously deleted task if no active-flow filename conflict exists.
    pub fn restore_task(&self, task: &Task) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let blocking_statuses = [TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done];

        if self
            .storage
            .find_task_by_exact_file_name_in_statuses(&task.file_name, &blocking_statuses)?
            .is_some()
        {
            return Err(ServiceError::TaskAlreadyExists(task.title.clone()));
        }

        self.enforce_status_limit(task.status)?;
        Ok(self.storage.write_task(task).map(|_| task.clone())?)
    }

    /// Overwrites title/type/details of the resolved task in place.
    ///
    /// Editing always clears any discard note so moving a previously discarded task
    /// back into active flow does not keep stale "won't do" context.
    pub fn edit_task(
        &self,
        query: &str,
        title: String,
        task_type: crate::domain::TaskType,
        details: String,
    ) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let normalized_details = normalize_escaped_newlines(&details);
        validate_title(&title).map_err(validation_error)?;
        validate_details(&normalized_details, false).map_err(validation_error)?;

        let mut task = self.resolve_task(query)?;
        task.title = title.trim().to_string();
        task.task_type = task_type;
        task.discard_note = None;
        task.details = normalized_details.trim_end().to_string();
        task.updated_at = Utc::now();
        Ok(self.storage.update_task(&task)?)
    }

    /// Resolves a user query against all visible tasks.
    fn resolve_task(&self, query: &str) -> Result<Task, ServiceError> {
        let all = self.storage.list_tasks(None, None)?;
        resolve_query(&all, query)
    }

    /// Enforces configured WIP limits for todo and in-progress buckets.
    fn enforce_status_limit(&self, status: TaskStatus) -> Result<(), ServiceError> {
        let limits = self.config.limits;
        let current = self.storage.list_tasks(Some(status), None)?.len();
        match status {
            TaskStatus::Todo if current >= limits.todo => Err(ServiceError::StatusLimitReached(
                format!("todo tasks are limited to {}", limits.todo),
            )),
            TaskStatus::InProgress if current >= limits.in_progress => {
                Err(ServiceError::StatusLimitReached(format!(
                    "in-progress tasks are limited to {}",
                    limits.in_progress
                )))
            }
            _ => Ok(()),
        }
    }
}

/// Converts domain validation failures into service-layer validation errors.
fn validation_error(err: DomainError) -> ServiceError {
    match err {
        DomainError::ValidationError(msg) => ServiceError::ValidationError(msg),
        other => ServiceError::ValidationError(other.to_string()),
    }
}
