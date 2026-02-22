use crate::domain::{Task, TaskStatus, TaskType, normalize_file_name};
use crate::storage::{Storage, StorageError};
use chrono::{DateTime, Utc};
use std::fs;

impl Storage {
    pub fn list_tasks(
        &self,
        status: Option<TaskStatus>,
        task_type: Option<TaskType>,
    ) -> Result<Vec<Task>, StorageError> {
        self.require_layout()?;
        let mut tasks = Vec::new();

        match status {
            Some(s) => {
                self.collect_bucket_tasks(s, task_type, &mut tasks)?;
            }
            None => {
                self.collect_bucket_tasks(TaskStatus::Todo, task_type, &mut tasks)?;
                self.collect_bucket_tasks(TaskStatus::InProgress, task_type, &mut tasks)?;
                self.collect_bucket_tasks(TaskStatus::Done, task_type, &mut tasks)?;
                self.collect_bucket_tasks(TaskStatus::Discard, task_type, &mut tasks)?;
            }
        }

        tasks.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| a.title.cmp(&b.title))
        });
        Ok(tasks)
    }

    pub fn find_task_by_exact_file_name(
        &self,
        file_name: &str,
    ) -> Result<Option<Task>, StorageError> {
        let md_name = format!("{file_name}.md");
        for status in [
            TaskStatus::Todo,
            TaskStatus::InProgress,
            TaskStatus::Done,
            TaskStatus::Discard,
        ] {
            let path = self.bucket_path(status).join(&md_name);
            if path.is_file() {
                return Ok(Some(self.parse_task_file(&path, status)?));
            }
        }
        Ok(None)
    }

    pub fn write_task(&self, task: &Task) -> Result<(), StorageError> {
        let path = self
            .bucket_path(task.status)
            .join(format!("{}.md", task.file_name));
        fs::create_dir_all(self.bucket_path(task.status))?;
        let content = self.render_task_markdown(task);
        fs::write(path, content)?;
        Ok(())
    }

    pub fn create_task(
        &self,
        title: &str,
        status: TaskStatus,
        task_type: TaskType,
        details: &str,
        now: DateTime<Utc>,
    ) -> Result<Task, StorageError> {
        let file_name = normalize_file_name(title)?;
        let task = Task {
            title: title.trim().to_string(),
            file_name,
            status,
            task_type,
            details: details.trim_end().to_string(),
            created_at: now,
            updated_at: now,
        };

        fs::create_dir_all(self.bucket_path(status))?;
        let content = self.render_task_markdown(&task);
        fs::write(self.task_path(&task), content)?;
        Ok(task)
    }

    pub fn update_task(&self, task: &Task) -> Result<Task, StorageError> {
        fs::create_dir_all(self.bucket_path(task.status))?;
        let content = self.render_task_markdown(task);
        fs::write(self.task_path(task), content)?;
        Ok(task.clone())
    }

    pub fn move_task(
        &self,
        task: &Task,
        new_status: TaskStatus,
        updated_at: DateTime<Utc>,
    ) -> Result<Task, StorageError> {
        let old_path = self.task_path(task);
        let mut updated = task.clone();
        updated.status = new_status;
        updated.updated_at = updated_at;

        fs::create_dir_all(self.bucket_path(new_status))?;
        let new_path = self.task_path(&updated);
        if old_path.exists() {
            fs::rename(&old_path, &new_path)?;
        }
        self.update_task(&updated)
    }

    pub fn delete_task(&self, task: &Task) -> Result<(), StorageError> {
        let path = self.task_path(task);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn read_task_content(&self, task: &Task) -> Result<String, StorageError> {
        Ok(fs::read_to_string(self.task_path(task))?)
    }
}
