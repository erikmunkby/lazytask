use super::query::resolve_query;
use super::{CreateTaskInput, ServiceError, TaskService};
use crate::config;
use crate::domain::{
    DomainError, Task, TaskStatus, normalize_escaped_newlines, normalize_file_name,
    parse_learning_lines, validate_details, validate_discard_note, validate_title,
};
use chrono::Utc;

impl TaskService {
    pub fn init(&self) -> Result<(), ServiceError> {
        config::ensure_default_file(&self.config)?;
        self.storage.ensure_layout()?;
        self.storage.ensure_agent_prompt_guidance()?;
        Ok(())
    }

    pub fn list_tasks(
        &self,
        status: Option<TaskStatus>,
        task_type: Option<crate::domain::TaskType>,
    ) -> Result<Vec<Task>, ServiceError> {
        Ok(self.storage.list_tasks(status, task_type)?)
    }

    pub fn get_tasks(&self, queries: &[String]) -> Result<Vec<Task>, ServiceError> {
        self.storage.require_layout()?;
        let all = self.storage.list_tasks(None, None)?;
        let mut out = Vec::new();

        for query in queries {
            out.push(resolve_query(&all, query)?);
        }

        Ok(out)
    }

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

    pub fn done_task_without_learning(&self, query: &str) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let task = self.resolve_task(query)?;

        Ok(self
            .storage
            .move_task(&task, TaskStatus::Done, Utc::now())?)
    }

    pub fn discard_task(&self, query: &str) -> Result<Task, ServiceError> {
        self.discard_task_internal(query, None)
    }

    pub fn discard_task_with_note(
        &self,
        query: &str,
        discard_note: &str,
    ) -> Result<Task, ServiceError> {
        let discard_note = validate_discard_note(discard_note).map_err(validation_error)?;
        self.discard_task_internal(query, Some(discard_note))
    }

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

    pub fn delete_task(&self, query: &str) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let task = self.resolve_task(query)?;

        self.storage.delete_task(&task)?;
        Ok(task)
    }

    pub fn delete_task_exact(&self, task: &Task) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        self.storage.delete_task(task)?;
        Ok(task.clone())
    }

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

    fn resolve_task(&self, query: &str) -> Result<Task, ServiceError> {
        let all = self.storage.list_tasks(None, None)?;
        resolve_query(&all, query)
    }

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

fn validation_error(err: DomainError) -> ServiceError {
    match err {
        DomainError::ValidationError(msg) => ServiceError::ValidationError(msg),
        other => ServiceError::ValidationError(other.to_string()),
    }
}
