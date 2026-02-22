use crate::domain::{Task, TaskStatus, TaskType, format_utc, parse_utc};
use crate::storage::{Storage, StorageError};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;
use std::str::FromStr;

impl Storage {
    pub(super) fn collect_bucket_tasks(
        &self,
        status: TaskStatus,
        task_type: Option<TaskType>,
        into: &mut Vec<Task>,
    ) -> Result<(), StorageError> {
        let bucket = self.bucket_path(status);
        if !bucket.exists() {
            return Ok(());
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
            if task_type.is_some_and(|kind| task.task_type != kind) {
                continue;
            }
            into.push(task);
        }

        Ok(())
    }

    pub fn parse_task_file(
        &self,
        path: &Path,
        fallback_status: TaskStatus,
    ) -> Result<Task, StorageError> {
        let contents = fs::read_to_string(path)?;
        let mut lines = contents.lines();

        let first_line = lines.next().unwrap_or("");
        let title = if let Some(stripped) = first_line.strip_prefix("# ") {
            stripped.trim().to_string()
        } else {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("untitled")
                .to_string()
        };

        let mut status = fallback_status;
        let mut task_type = TaskType::Task;
        let mut created_at: Option<DateTime<Utc>> = None;
        let mut updated_at: Option<DateTime<Utc>> = None;
        let mut details = Vec::new();
        let mut in_details = false;

        for line in lines {
            if in_details {
                if let Some(detail_line) = line.strip_prefix("  ") {
                    details.push(detail_line.to_string());
                } else if line.is_empty() {
                    details.push(String::new());
                } else {
                    details.push(line.to_string());
                }
                continue;
            }

            if let Some(value) = line.strip_prefix("status:") {
                if let Ok(parsed) = TaskStatus::from_str(value.trim()) {
                    status = parsed;
                }
                continue;
            }

            if let Some(value) = line.strip_prefix("type:") {
                if let Ok(parsed) = TaskType::from_str(value.trim()) {
                    task_type = parsed;
                }
                continue;
            }

            if let Some(value) = line.strip_prefix("created:") {
                if let Ok(parsed) = parse_utc(value.trim()) {
                    created_at = Some(parsed);
                }
                continue;
            }

            if let Some(value) = line.strip_prefix("updated:") {
                if let Ok(parsed) = parse_utc(value.trim()) {
                    updated_at = Some(parsed);
                }
                continue;
            }

            if line.trim() == "details:" {
                in_details = true;
            }
        }

        let details = details.join("\n").trim_end().to_string();

        let now = Utc::now();
        let file_name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| StorageError::Parse("invalid task file name".to_string()))?
            .to_string();

        Ok(Task {
            title,
            file_name,
            status,
            task_type,
            details,
            created_at: created_at.unwrap_or(now),
            updated_at: updated_at.unwrap_or(now),
        })
    }

    pub(super) fn render_task_markdown(&self, task: &Task) -> String {
        let mut out = String::new();
        out.push_str(&format!("# {}\n", task.title));
        out.push_str(&format!("status: {}\n", task.status));
        out.push_str(&format!("type: {}\n", task.task_type));
        out.push_str(&format!("created: {}\n", format_utc(task.created_at)));
        out.push_str(&format!("updated: {}\n", format_utc(task.updated_at)));
        out.push_str("details:\n");
        for line in task.details.lines() {
            out.push_str(&format!("  {}\n", line));
        }
        out
    }
}
