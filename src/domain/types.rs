use super::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub const TITLE_CHAR_LIMIT: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Discard,
}

impl TaskStatus {
    /// Returns the canonical storage/CLI string for this status.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in-progress",
            Self::Done => "done",
            Self::Discard => "discard",
        }
    }
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TaskStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "todo" => Ok(Self::Todo),
            "in-progress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            "discard" => Ok(Self::Discard),
            other => Err(DomainError::ParseError(format!(
                "unknown task status: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    Task,
    Bug,
}

impl TaskType {
    /// Returns the canonical storage/CLI string for this task kind.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Bug => "bug",
        }
    }
}

impl Display for TaskType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TaskType {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "task" => Ok(Self::Task),
            "bug" => Ok(Self::Bug),
            other => Err(DomainError::ParseError(format!(
                "unknown task type: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub title: String,
    pub file_name: String,
    pub status: TaskStatus,
    pub task_type: TaskType,
    pub discard_note: Option<String>,
    pub details: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
