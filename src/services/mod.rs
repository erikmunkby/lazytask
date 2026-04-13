mod clipboard;
mod editor;
mod errors;
mod learning;
mod query;
mod retention;
mod tasks;

use crate::config::AppConfig;
use crate::storage::Storage;
use serde::Serialize;

pub use clipboard::PasteResult;
pub use errors::ServiceError;

#[derive(Debug, Clone)]
pub struct CreateTaskInput {
    pub title: String,
    pub task_type: crate::domain::TaskType,
    pub details: String,
    pub start: bool,
    pub require_details: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnResult {
    pub entries: Vec<LearnEntry>,
    pub instructions: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnEntry {
    pub date: String,
    pub learnings: String,
}

#[derive(Debug, Clone)]
pub struct TaskService {
    pub(crate) storage: Storage,
    pub(crate) config: AppConfig,
}

impl TaskService {
    /// Builds a service facade with storage bound to the provided app config.
    pub fn new(config: AppConfig) -> Self {
        Self {
            storage: Storage::from_app_config(&config),
            config,
        }
    }

    /// Reads the system clipboard and saves any image as PNG under `.tasks/assets/`.
    pub fn paste_from_clipboard(&self) -> Result<PasteResult, ServiceError> {
        clipboard::paste_from_clipboard(&self.storage.tasks_root())
    }

    /// Copies the given text to the system clipboard.
    pub fn copy_to_clipboard(&self, text: &str) -> Result<(), ServiceError> {
        clipboard::copy_to_clipboard(text)
    }
}

#[cfg(test)]
mod tests;
