use super::query::resolve_query;
use super::{CreateTaskInput, ServiceError, TaskService};
use crate::config;
use crate::domain::{
    DomainError, Task, TaskStatus, normalize_escaped_newlines, normalize_file_name,
    parse_learning_lines, validate_details, validate_title,
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
        if self
            .storage
            .find_task_by_exact_file_name(&file_name)?
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
        let all = self.storage.list_tasks(None, None)?;
        let task = resolve_query(&all, query)?;
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
        let all = self.storage.list_tasks(None, None)?;
        let task = resolve_query(&all, query)?;

        let now = Utc::now();
        let moved = self.storage.move_task(&task, TaskStatus::Done, now)?;
        self.storage
            .append_learning(now, &moved.title, &learning_lines)?;

        Ok(moved)
    }

    pub fn done_task_without_learning(&self, query: &str) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let all = self.storage.list_tasks(None, None)?;
        let task = resolve_query(&all, query)?;

        Ok(self
            .storage
            .move_task(&task, TaskStatus::Done, Utc::now())?)
    }

    pub fn discard_task(&self, query: &str) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let all = self.storage.list_tasks(None, None)?;
        let task = resolve_query(&all, query)?;

        Ok(self
            .storage
            .move_task(&task, TaskStatus::Discard, Utc::now())?)
    }

    pub fn delete_task(&self, query: &str) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        let all = self.storage.list_tasks(None, None)?;
        let task = resolve_query(&all, query)?;

        self.storage.delete_task(&task)?;
        Ok(task)
    }

    pub fn restore_task(&self, task: &Task) -> Result<Task, ServiceError> {
        self.storage.require_layout()?;
        if self
            .storage
            .find_task_by_exact_file_name(&task.file_name)?
            .is_some()
        {
            return Err(ServiceError::TaskAlreadyExists(task.title.clone()));
        }

        self.enforce_status_limit(task.status)?;
        Ok(self.storage.write_task(task).map(|_| task.clone())?)
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
