mod error;
mod file_name;
mod learning;
mod time;
mod types;
mod validation;

pub use error::DomainError;
pub use file_name::normalize_file_name;
pub use learning::{normalize_escaped_newlines, parse_learning_lines};
pub use time::{format_local_human, format_relative, format_utc, parse_utc};
pub use types::{TITLE_CHAR_LIMIT, Task, TaskStatus, TaskType};
pub use validation::{validate_details, validate_title};

#[cfg(test)]
mod tests;
