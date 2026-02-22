use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("parse error: {0}")]
    ParseError(String),
}
