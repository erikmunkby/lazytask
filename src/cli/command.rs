use crate::domain::TaskType;
use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(name = "lt", version, about = "AI-first task manager")]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub(super) enum Commands {
    /// Initialize lazytask in the current project.
    Init {
        /// Rewrite generated config and guidance defaults without touching `.tasks/`.
        #[arg(long, default_value_t = false)]
        upgrade: bool,
    },
    /// List tasks, optionally filtered by type.
    List {
        /// Filter by task or bug.
        #[arg(long = "type", value_enum)]
        task_type: Option<TaskType>,
        /// Include completed tasks.
        #[arg(long, default_value_t = false)]
        show_done: bool,
    },
    /// Get full details for one or more tasks by title.
    Get {
        #[arg(required = true)]
        query: Vec<String>,
    },
    /// Create a new task or bug.
    Create {
        #[arg(long)]
        title: String,
        /// Task or bug.
        #[arg(long = "type", value_enum)]
        task_type: TaskType,
        #[arg(long, allow_hyphen_values = true)]
        details: String,
        /// Immediately move the task to in-progress.
        #[arg(long, default_value_t = false)]
        start: bool,
    },
    /// Move a task to in-progress.
    Start { query: String },
    /// Mark a task as done.
    Done { query: String },
    /// Discard a task that won't be done.
    Discard {
        query: String,
        /// Short explanation for why the task is being discarded.
        #[arg(long, allow_hyphen_values = true)]
        discard_note: String,
    },
    /// Edit a task's title, type, or details.
    Edit {
        query: String,
        /// New title. Keeps current if omitted.
        #[arg(long)]
        title: Option<String>,
        /// New type. Keeps current if omitted.
        #[arg(long = "type", value_enum)]
        task_type: Option<TaskType>,
        /// New details. Keeps current if omitted.
        #[arg(long, allow_hyphen_values = true)]
        details: Option<String>,
    },
    /// Permanently delete a task.
    Delete { query: String },
    /// Record or review learnings from completed tasks.
    Learn {
        /// A learning to record.
        #[arg(long, allow_hyphen_values = true)]
        learning: Option<String>,
        /// Review unconsumed learnings.
        #[arg(long)]
        review: bool,
        #[arg(long, hide = true)]
        finished: bool,
    },
}

#[derive(Debug, Serialize)]
pub(super) struct TaskData {
    pub title: String,
    pub status: String,
    pub task_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discard_note: Option<String>,
    pub details: String,
    pub updated: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step: Option<String>,
}
