use crate::config::ConfigError;
use crate::domain::DomainError;
use crate::storage::StorageError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("tasks root missing")]
    TasksRootMissing,
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("task query is ambiguous: {query}")]
    TaskAmbiguous { query: String, matches: Vec<String> },
    #[error("task already exists: {0}")]
    TaskAlreadyExists(String),
    #[error("status limit reached: {0}")]
    StatusLimitReached(String),
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("parse error: {0}")]
    ParseError(String),
}

impl From<StorageError> for ServiceError {
    fn from(value: StorageError) -> Self {
        match value {
            StorageError::TasksRootMissing => ServiceError::TasksRootMissing,
            StorageError::Io(err) => ServiceError::Io(err.to_string()),
            StorageError::Parse(msg) => ServiceError::ParseError(msg),
            StorageError::Domain(err) => match err {
                DomainError::ValidationError(msg) => ServiceError::ValidationError(msg),
                DomainError::ParseError(msg) => ServiceError::ParseError(msg),
            },
        }
    }
}

impl From<ConfigError> for ServiceError {
    fn from(value: ConfigError) -> Self {
        match value {
            ConfigError::Io(err) => ServiceError::Io(err.to_string()),
            ConfigError::Parse(msg) => ServiceError::ParseError(msg),
        }
    }
}
