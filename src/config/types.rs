use super::prompts::{DEFAULT_PROMPT_CONFIG, PromptConfig};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitsConfig {
    pub todo: usize,
    pub in_progress: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HintsConfig {
    pub learn_threshold: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageLayoutConfig {
    pub tasks_dir: &'static str,
    pub todo_dir: &'static str,
    pub in_progress_dir: &'static str,
    pub done_dir: &'static str,
    pub discard_dir: &'static str,
    pub learnings_file: &'static str,
    pub agents_file: &'static str,
    pub claude_file: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub workspace_root: PathBuf,
    pub limits: LimitsConfig,
    pub hints: HintsConfig,
    pub storage_layout: StorageLayoutConfig,
    pub prompts: PromptConfig,
    pub(crate) config_file_name: &'static str,
}

impl AppConfig {
    pub fn config_path(&self) -> PathBuf {
        self.workspace_root.join(self.config_file_name)
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InternalConfig {
    pub config_file_name: &'static str,
    pub default_limits: LimitsConfig,
    pub default_hints: HintsConfig,
    pub storage_layout: StorageLayoutConfig,
    pub prompts: PromptConfig,
}

pub(crate) const INTERNAL_CONFIG: InternalConfig = InternalConfig {
    config_file_name: "lazytask.toml",
    default_limits: LimitsConfig {
        todo: 20,
        in_progress: 3,
    },
    default_hints: HintsConfig {
        learn_threshold: 35,
    },
    storage_layout: StorageLayoutConfig {
        tasks_dir: ".tasks",
        todo_dir: "todo",
        in_progress_dir: "in-progress",
        done_dir: "done",
        discard_dir: "discard",
        learnings_file: "LEARNINGS.md",
        agents_file: "AGENTS.md",
        claude_file: "CLAUDE.md",
    },
    prompts: DEFAULT_PROMPT_CONFIG,
};
