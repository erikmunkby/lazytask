use super::schema::{
    ConfigKeySchema, USER_CONFIG_SCHEMA, default_usize, render_default_config_body,
};
use super::types::{
    AppConfig, ConfigError, HintsConfig, INTERNAL_CONFIG, LimitsConfig, RetentionConfig,
};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Table, TableLike, value};

#[derive(Debug, Deserialize, Default)]
struct UserConfig {
    limits: Option<UserLimitsConfig>,
    hints: Option<UserHintsConfig>,
    retention: Option<UserRetentionConfig>,
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

#[derive(Debug, Deserialize)]
struct UserRetentionConfig {
    done_discard_ttl_days: Option<usize>,
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
            .unwrap_or(default_usize("limits", "todo")),
        in_progress: user
            .limits
            .as_ref()
            .and_then(|limits| limits.in_progress)
            .unwrap_or(default_usize("limits", "in_progress")),
    };
    let hints = HintsConfig {
        learn_threshold: user
            .hints
            .as_ref()
            .and_then(|hints| hints.learn_threshold)
            .unwrap_or(default_usize("hints", "learn_threshold")),
    };
    let retention = RetentionConfig {
        done_discard_ttl_days: user
            .retention
            .as_ref()
            .and_then(|retention| retention.done_discard_ttl_days)
            .unwrap_or(default_usize("retention", "done_discard_ttl_days")),
    };

    validate_min(limits.todo, "limits", "todo")?;
    validate_min(limits.in_progress, "limits", "in_progress")?;
    validate_min(hints.learn_threshold, "hints", "learn_threshold")?;
    validate_min(
        retention.done_discard_ttl_days,
        "retention",
        "done_discard_ttl_days",
    )?;

    Ok(AppConfig {
        workspace_root,
        limits,
        hints,
        retention,
        storage_layout: INTERNAL_CONFIG.storage_layout,
        prompts: INTERNAL_CONFIG.prompts,
        config_file_name: INTERNAL_CONFIG.config_file_name,
    })
}

pub fn ensure_default_file(config: &AppConfig) -> Result<(), ConfigError> {
    let path = config.config_path();
    if !path.exists() {
        fs::write(path, render_default_config_body())?;
        return Ok(());
    }

    let original = fs::read_to_string(&path)?;
    if let Some(updated) = backfill_missing_keys(&original)? {
        fs::write(path, updated)?;
    }
    Ok(())
}

fn backfill_missing_keys(source: &str) -> Result<Option<String>, ConfigError> {
    let mut document = source
        .parse::<DocumentMut>()
        .map_err(|err| ConfigError::Parse(format!("invalid lazytask.toml: {err}")))?;
    let mut changed = false;

    for section in USER_CONFIG_SCHEMA {
        if !document.contains_key(section.name) {
            let mut table = Table::new();
            for key in section.keys {
                insert_default_key(&mut table, key);
            }
            document.insert(section.name, Item::Table(table));
            changed = true;
            continue;
        }

        let section_table = document
            .get_mut(section.name)
            .and_then(Item::as_table_like_mut)
            .ok_or_else(|| {
                ConfigError::Parse(format!(
                    "invalid lazytask.toml: {} must be a table",
                    section.name
                ))
            })?;

        for key in section.keys {
            if !section_table.contains_key(key.name) {
                insert_default_key(section_table, key);
                changed = true;
            }
        }
    }

    if changed {
        Ok(Some(document.to_string()))
    } else {
        Ok(None)
    }
}

fn insert_default_key(table: &mut (impl TableLike + ?Sized), key: &ConfigKeySchema) {
    table.insert(key.name, value(key.default as i64));
    if let Some(item) = table.get_mut(key.name)
        && let Some(value) = item.as_value_mut()
    {
        value
            .decor_mut()
            .set_suffix(format!(" # {}", key.description));
    }
}

fn read_user_config(path: &Path) -> Result<UserConfig, ConfigError> {
    if !path.exists() {
        return Ok(UserConfig::default());
    }

    let contents = fs::read_to_string(path)?;
    toml::from_str(&contents)
        .map_err(|err| ConfigError::Parse(format!("invalid lazytask.toml: {err}")))
}

fn validate_min(value: usize, section: &str, key: &str) -> Result<(), ConfigError> {
    let min = super::schema::min_usize(section, key);
    if value < min {
        return Err(ConfigError::Parse(format!(
            "invalid lazytask.toml: {section}.{key} must be >= {min}"
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
