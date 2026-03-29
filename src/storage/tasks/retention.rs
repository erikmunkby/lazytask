use crate::domain::{Task, TaskStatus};
use crate::storage::{Storage, StorageError};
use chrono::{DateTime, Utc};
use std::fs;

impl Storage {
    /// Deletes `done` and `discard` tasks whose `updated_at` is at or before `cutoff`.
    ///
    /// Active buckets are never touched. Returns the deleted tasks so callers can
    /// perform additional cleanup (e.g. removing referenced asset files).
    pub fn delete_terminal_tasks_updated_before(
        &self,
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<Task>, StorageError> {
        if !self.tasks_root().exists() {
            return Ok(Vec::new());
        }

        let mut deleted = Vec::new();
        for status in [TaskStatus::Done, TaskStatus::Discard] {
            let bucket = self.bucket_path(status);
            if !bucket.is_dir() {
                continue;
            }

            for entry in fs::read_dir(bucket)? {
                let entry = entry?;
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                    continue;
                }

                let task = self.parse_task_file(&path, status)?;
                if task.updated_at <= cutoff {
                    fs::remove_file(path)?;
                    deleted.push(task);
                }
            }
        }

        Ok(deleted)
    }
}
