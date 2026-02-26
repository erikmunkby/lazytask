mod editor;
mod errors;
mod learning;
mod query;
mod retention;
mod tasks;

use crate::config::AppConfig;
use crate::storage::Storage;
use serde::Serialize;

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
    pub title: String,
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
}

#[cfg(test)]
mod tests;
