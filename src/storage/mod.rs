mod agent_guidance;
mod learning;
mod tasks;

use crate::config::{AppConfig, PromptConfig, StorageLayoutConfig};
use crate::domain::{DomainError, Task, TaskStatus};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LearningEntry {
    pub timestamp: DateTime<Utc>,
    pub task_title: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("tasks root missing")]
    TasksRootMissing,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error(transparent)]
    Domain(#[from] DomainError),
}

#[derive(Debug, Clone)]
pub struct Storage {
    pub workspace_root: PathBuf,
    layout: StorageLayoutConfig,
    prompts: PromptConfig,
}

impl Storage {
    /// Creates a storage handle rooted at an explicit workspace path.
    pub fn from_path(path: PathBuf, layout: StorageLayoutConfig, prompts: PromptConfig) -> Self {
        Self {
            workspace_root: path,
            layout,
            prompts,
        }
    }

    /// Creates storage from the runtime app config.
    pub fn from_app_config(config: &AppConfig) -> Self {
        Self::from_path(
            config.workspace_root.clone(),
            config.storage_layout,
            config.prompts,
        )
    }

    /// Returns the `.tasks` root path for this workspace.
    pub fn tasks_root(&self) -> PathBuf {
        self.workspace_root.join(self.layout.tasks_dir)
    }

    /// Returns the path to the shared learnings markdown file.
    pub fn learnings_path(&self) -> PathBuf {
        self.tasks_root().join(self.layout.learnings_file)
    }

    /// Creates all task status buckets if they do not already exist.
    pub fn ensure_layout(&self) -> Result<(), StorageError> {
        fs::create_dir_all(self.bucket_path(TaskStatus::Todo))?;
        fs::create_dir_all(self.bucket_path(TaskStatus::InProgress))?;
        fs::create_dir_all(self.bucket_path(TaskStatus::Done))?;
        fs::create_dir_all(self.bucket_path(TaskStatus::Discard))?;
        Ok(())
    }

    /// Verifies required task layout exists before read/write operations.
    ///
    /// The discard bucket is auto-created for older workspaces that predate it.
    pub fn require_layout(&self) -> Result<(), StorageError> {
        if !self.tasks_root().is_dir()
            || !self.bucket_path(TaskStatus::Todo).is_dir()
            || !self.bucket_path(TaskStatus::InProgress).is_dir()
            || !self.bucket_path(TaskStatus::Done).is_dir()
        {
            return Err(StorageError::TasksRootMissing);
        }

        if !self.bucket_path(TaskStatus::Discard).is_dir() {
            fs::create_dir_all(self.bucket_path(TaskStatus::Discard))?;
        }
        Ok(())
    }

    /// Returns the directory for a specific task status bucket.
    pub fn bucket_path(&self, status: TaskStatus) -> PathBuf {
        match status {
            TaskStatus::Todo => self.tasks_root().join(self.layout.todo_dir),
            TaskStatus::InProgress => self.tasks_root().join(self.layout.in_progress_dir),
            TaskStatus::Done => self.tasks_root().join(self.layout.done_dir),
            TaskStatus::Discard => self.tasks_root().join(self.layout.discard_dir),
        }
    }

    /// Returns the canonical markdown path for a task based on status + file name.
    pub(crate) fn task_path(&self, task: &Task) -> PathBuf {
        self.bucket_path(task.status)
            .join(format!("{}.md", task.file_name))
    }
}

#[cfg(test)]
mod tests;
