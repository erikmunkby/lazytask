use super::prompts::render_prompts_section;
use super::schema::{
    ConfigKeySchema, USER_CONFIG_SCHEMA, default_usize, render_default_config_body,
};
use super::types::{
    AppConfig, ConfigError, HintsConfig, INTERNAL_CONFIG, LimitsConfig, PromptOverrides,
    RetentionConfig,
};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Table, TableLike, value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceSource {
    GitRoot,
    GitWorktree,
    EnvOverride,
    Cwd,
}

pub struct ResolvedWorkspace {
    pub root: PathBuf,
    pub source: WorkspaceSource,
}

/// Resolves the workspace root by checking `LAZYTASK_DIR`, then git.
pub fn resolve_workspace() -> Result<ResolvedWorkspace, ConfigError> {
    let current = std::env::current_dir()?;
    resolve_workspace_from(&current, std::env::var("LAZYTASK_DIR").ok())
}

/// Pure resolver — testable without env mutation.
pub(crate) fn resolve_workspace_from(
    cwd: &Path,
    lazytask_dir: Option<String>,
) -> Result<ResolvedWorkspace, ConfigError> {
    if let Some(dir) = lazytask_dir {
        let path = PathBuf::from(&dir);
        let root = if path.is_absolute() {
            path
        } else {
            cwd.join(path)
        };
        return Ok(ResolvedWorkspace {
            root,
            source: WorkspaceSource::EnvOverride,
        });
    }
    Ok(find_workspace_root_with_source(cwd))
}

/// Returns the git root for AGENTS.md placement, if a `.git` anchor exists.
pub fn find_git_root(start: &Path) -> Option<PathBuf> {
    let resolved = find_workspace_root_with_source(start);
    match resolved.source {
        WorkspaceSource::GitRoot | WorkspaceSource::GitWorktree => Some(resolved.root),
        WorkspaceSource::Cwd | WorkspaceSource::EnvOverride => None,
    }
}

#[derive(Debug, Deserialize, Default)]
struct UserConfig {
    limits: Option<UserLimitsConfig>,
    hints: Option<UserHintsConfig>,
    retention: Option<UserRetentionConfig>,
    prompts: Option<UserPromptsConfig>,
}

#[derive(Debug, Deserialize)]
struct UserPromptsConfig {
    done_reflection: Option<String>,
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
    cleanup_task_assets: Option<bool>,
}

/// Loads app config by resolving the workspace root from the current directory.
pub fn load_from_current_dir() -> Result<AppConfig, ConfigError> {
    let resolved = resolve_workspace()?;
    load_for_workspace_root(resolved.root)
}

/// Loads effective config for a known workspace root.
///
/// User overrides are merged onto internal defaults and validated against schema minima.
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
        cleanup_task_assets: user
            .retention
            .as_ref()
            .and_then(|retention| retention.cleanup_task_assets)
            .unwrap_or(true),
    };

    let prompt_overrides = PromptOverrides {
        done_reflection: user.prompts.and_then(|p| p.done_reflection),
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
        prompt_overrides,
        config_file_name: INTERNAL_CONFIG.config_file_name,
    })
}

/// Ensures `lazytask.toml` exists and contains all currently supported keys.
///
/// Existing values are preserved; only missing sections/keys are backfilled.
pub fn ensure_default_file(config: &AppConfig) -> Result<(), ConfigError> {
    ensure_default_file_with_upgrade(config, false)
}

/// Ensures `lazytask.toml` exists, with optional full overwrite in upgrade mode.
pub fn ensure_default_file_with_upgrade(
    config: &AppConfig,
    upgrade: bool,
) -> Result<(), ConfigError> {
    let path = config.config_path();
    if !path.exists() || upgrade {
        let full_default = format!(
            "{}{}",
            render_default_config_body(),
            render_prompts_section()
        );
        fs::write(path, full_default)?;
        return Ok(());
    }

    let original = fs::read_to_string(&path)?;
    if let Some(updated) = backfill_missing_keys(&original)? {
        fs::write(path, updated)?;
    }
    Ok(())
}

/// Inserts any schema keys missing from an existing config document.
///
/// Returns `Some(updated_toml)` only when the source needs changes.
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
            if section.name == "retention" {
                insert_bool_key(
                    &mut table,
                    "cleanup_task_assets",
                    true,
                    "delete referenced images when a task is deleted",
                );
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

        // Backfill boolean keys that live outside the usize-only schema.
        if section.name == "retention" && !section_table.contains_key("cleanup_task_assets") {
            insert_bool_key(
                section_table,
                "cleanup_task_assets",
                true,
                "delete referenced images when a task is deleted",
            );
            changed = true;
        }
    }

    // Backfill [prompts] via raw text to preserve triple-quoted multiline values
    // that toml_edit would mangle.
    let needs_prompts = !source.contains("[prompts]");

    if !changed && !needs_prompts {
        return Ok(None);
    }

    let mut result = if changed {
        document.to_string()
    } else {
        source.to_string()
    };

    if needs_prompts {
        result.push_str(&render_prompts_section());
    }

    Ok(Some(result))
}

/// Inserts a default scalar key and appends its inline description comment.
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

/// Inserts a boolean key with an inline description comment.
fn insert_bool_key(table: &mut (impl TableLike + ?Sized), name: &str, default: bool, desc: &str) {
    table.insert(name, value(default));
    if let Some(item) = table.get_mut(name)
        && let Some(v) = item.as_value_mut()
    {
        v.decor_mut().set_suffix(format!(" # {desc}"));
    }
}

/// Reads and deserializes user config if present, otherwise returns empty overrides.
fn read_user_config(path: &Path) -> Result<UserConfig, ConfigError> {
    if !path.exists() {
        return Ok(UserConfig::default());
    }

    let contents = fs::read_to_string(path)?;
    toml::from_str(&contents)
        .map_err(|err| ConfigError::Parse(format!("invalid lazytask.toml: {err}")))
}

/// Validates a numeric config value against schema minimum constraints.
fn validate_min(value: usize, section: &str, key: &str) -> Result<(), ConfigError> {
    let min = super::schema::min_usize(section, key);
    if value < min {
        return Err(ConfigError::Parse(format!(
            "invalid lazytask.toml: {section}.{key} must be >= {min}"
        )));
    }

    Ok(())
}

/// Walks up from `start` looking for `.git` (directory or worktree file).
fn find_workspace_root_with_source(start: &Path) -> ResolvedWorkspace {
    let mut cursor = start.to_path_buf();
    loop {
        let git_path = cursor.join(".git");
        if git_path.is_dir() {
            return ResolvedWorkspace {
                root: cursor,
                source: WorkspaceSource::GitRoot,
            };
        }
        if git_path.is_file()
            && let Some(main_root) = resolve_worktree_main_root(&git_path)
        {
            return ResolvedWorkspace {
                root: main_root,
                source: WorkspaceSource::GitWorktree,
            };
        }
        if !cursor.pop() {
            break;
        }
    }
    ResolvedWorkspace {
        root: start.to_path_buf(),
        source: WorkspaceSource::Cwd,
    }
}

/// Parses a worktree `.git` file and resolves back to the main repo root.
pub(crate) fn resolve_worktree_main_root(git_file: &Path) -> Option<PathBuf> {
    let content = fs::read_to_string(git_file).ok()?;
    let gitdir = content.strip_prefix("gitdir: ")?.trim();
    let gitdir_path = if Path::new(gitdir).is_absolute() {
        PathBuf::from(gitdir)
    } else {
        git_file.parent()?.join(gitdir)
    };
    // Canonicalize to resolve `..` segments from relative gitdir paths.
    let gitdir_path = gitdir_path.canonicalize().ok()?;
    let mut ancestor = gitdir_path.as_path();
    loop {
        if ancestor.file_name()?.to_str()? == "worktrees" {
            let dot_git = ancestor.parent()?;
            // Only accept standard `.git` dirs, not bare repos (e.g. `repo.git`).
            if dot_git.file_name()?.to_str()? != ".git" {
                return None;
            }
            return dot_git.parent().map(|p| p.to_path_buf());
        }
        ancestor = ancestor.parent()?;
    }
}
