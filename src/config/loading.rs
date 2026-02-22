use super::types::{AppConfig, ConfigError, HintsConfig, INTERNAL_CONFIG, LimitsConfig};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default)]
struct UserConfig {
    limits: Option<UserLimitsConfig>,
    hints: Option<UserHintsConfig>,
}

#[derive(Debug, Deserialize)]
struct UserLimitsConfig {
    todo: Option<usize>,
    in_progress: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct UserHintsConfig {
    learn_threshold: Option<usize>,
}

pub fn load_from_current_dir() -> Result<AppConfig, ConfigError> {
    let current = std::env::current_dir()?;
    load_for_workspace_root(find_workspace_root(&current))
}

pub fn load_for_workspace_root(workspace_root: impl AsRef<Path>) -> Result<AppConfig, ConfigError> {
    let workspace_root = workspace_root.as_ref().to_path_buf();
    let config_path = workspace_root.join(INTERNAL_CONFIG.config_file_name);
    let user = read_user_config(&config_path)?;

    let limits = LimitsConfig {
        todo: user
            .limits
            .as_ref()
            .and_then(|limits| limits.todo)
            .unwrap_or(INTERNAL_CONFIG.default_limits.todo),
        in_progress: user
            .limits
            .as_ref()
            .and_then(|limits| limits.in_progress)
            .unwrap_or(INTERNAL_CONFIG.default_limits.in_progress),
    };
    let hints = HintsConfig {
        learn_threshold: user
            .hints
            .as_ref()
            .and_then(|hints| hints.learn_threshold)
            .unwrap_or(INTERNAL_CONFIG.default_hints.learn_threshold),
    };

    validate_gte_one(limits.todo, "limits.todo")?;
    validate_gte_one(limits.in_progress, "limits.in_progress")?;
    validate_gte_one(hints.learn_threshold, "hints.learn_threshold")?;

    Ok(AppConfig {
        workspace_root,
        limits,
        hints,
        storage_layout: INTERNAL_CONFIG.storage_layout,
        prompts: INTERNAL_CONFIG.prompts,
        config_file_name: INTERNAL_CONFIG.config_file_name,
    })
}

pub fn ensure_default_file(config: &AppConfig) -> Result<(), ConfigError> {
    let path = config.config_path();
    if path.exists() {
        return Ok(());
    }

    fs::write(path, default_config_body())?;
    Ok(())
}

fn default_config_body() -> String {
    format!(
        "[limits]\ntodo = {} # max todo tasks\nin_progress = {} # max in-progress tasks\n\n[hints]\nlearn_threshold = {} # show `lt learn` hint after this many LEARNINGS.md lines\n",
        INTERNAL_CONFIG.default_limits.todo,
        INTERNAL_CONFIG.default_limits.in_progress,
        INTERNAL_CONFIG.default_hints.learn_threshold
    )
}

fn read_user_config(path: &Path) -> Result<UserConfig, ConfigError> {
    if !path.exists() {
        return Ok(UserConfig::default());
    }

    let contents = fs::read_to_string(path)?;
    toml::from_str(&contents)
        .map_err(|err| ConfigError::Parse(format!("invalid lazytask.toml: {err}")))
}

fn validate_gte_one(value: usize, key: &str) -> Result<(), ConfigError> {
    if value < 1 {
        return Err(ConfigError::Parse(format!(
            "invalid lazytask.toml: {key} must be >= 1"
        )));
    }

    Ok(())
}

pub(crate) fn find_workspace_root(start: &Path) -> PathBuf {
    let mut cursor = start.to_path_buf();

    loop {
        if cursor.join(".git").exists() {
            return cursor;
        }

        if !cursor.pop() {
            break;
        }
    }

    start.to_path_buf()
}
