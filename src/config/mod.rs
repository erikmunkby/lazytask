mod loading;
mod prompts;
mod schema;
mod types;

pub use loading::{
    ensure_default_file, ensure_default_file_with_upgrade, load_for_workspace_root,
    load_from_current_dir,
};
pub use prompts::{PromptConfig, markdown_for_key};
pub use types::{
    AppConfig, ConfigError, HintsConfig, LimitsConfig, RetentionConfig, StorageLayoutConfig,
};

#[cfg(test)]
mod tests;
