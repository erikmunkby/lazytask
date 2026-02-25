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
    Init,
    List {
        #[arg(long = "type", value_enum)]
        task_type: Option<TaskType>,
        #[arg(long, default_value_t = false)]
        show_done: bool,
    },
    Get {
        #[arg(required = true)]
        query: Vec<String>,
    },
    Create {
        #[arg(long)]
        title: String,
        #[arg(long = "type", value_enum)]
        task_type: TaskType,
        #[arg(long, allow_hyphen_values = true)]
        details: String,
        #[arg(long, default_value_t = false)]
        start: bool,
    },
    Start {
        query: String,
    },
    Done {
        query: String,
        #[arg(long)]
        learning: String,
    },
    Discard {
        query: String,
        #[arg(long, allow_hyphen_values = true)]
        discard_note: String,
    },
    Delete {
        query: String,
    },
    Learn {
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
}
