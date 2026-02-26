use crate::domain::TaskStatus;
use crate::storage::{Storage, StorageError};
use chrono::{DateTime, Utc};
use std::fs;

impl Storage {
    /// Deletes `done` and `discard` tasks whose `updated_at` is at or before `cutoff`.
    ///
    /// Active buckets are never touched.
    pub fn delete_terminal_tasks_updated_before(
        &self,
        cutoff: DateTime<Utc>,
    ) -> Result<usize, StorageError> {
        if !self.tasks_root().exists() {
            return Ok(0);
        }

        let mut deleted = 0;
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
                    deleted += 1;
                }
            }
        }

        Ok(deleted)
    }
}
