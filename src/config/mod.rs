mod loading;
mod prompts;
mod schema;
mod types;

pub use loading::{
    ResolvedWorkspace, WorkspaceSource, ensure_default_file, ensure_default_file_with_upgrade,
    find_git_root, load_for_workspace_root, load_from_current_dir, resolve_workspace,
};
pub use prompts::{PromptConfig, markdown_for_key, resolve_done_reflection};
pub use types::{
    AppConfig, ConfigError, HintsConfig, LimitsConfig, PromptOverrides, RetentionConfig,
    StorageLayoutConfig,
};

#[cfg(test)]
mod tests;
