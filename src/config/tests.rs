use super::*;
use crate::config::schema::default_usize;
use loading::{WorkspaceSource, resolve_workspace_from};
use std::fs;
use tempfile::TempDir;

#[test]
fn load_uses_defaults_when_file_is_missing() {
    let temp = TempDir::new().unwrap();

    let config = load_for_workspace_root(temp.path()).unwrap();

    assert_eq!(config.limits.todo, default_usize("limits", "todo"));
    assert_eq!(
        config.limits.in_progress,
        default_usize("limits", "in_progress")
    );
    assert_eq!(
        config.hints.learn_threshold,
        default_usize("hints", "learn_threshold")
    );
    assert_eq!(
        config.retention.done_discard_ttl_days,
        default_usize("retention", "done_discard_ttl_days")
    );
    assert_eq!(config.storage_layout.tasks_dir, ".tasks");
    assert_eq!(config.storage_layout.learnings_file, "LEARNINGS.md");
    assert_eq!(
        config.prompts.learn_threshold_hint_key,
        "learn_threshold_hint"
    );
    assert_eq!(config.prompt_overrides.done_reflection, None);
    assert_eq!(config.config_path(), temp.path().join("lazytask.toml"));
}

#[test]
fn load_merges_overrides_with_defaults() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("lazytask.toml"),
        "[limits]\ntodo = 7\n\n[hints]\nlearn_threshold = 10\n\n[retention]\ndone_discard_ttl_days = 12\n",
    )
    .unwrap();

    let config = load_for_workspace_root(temp.path()).unwrap();

    assert_eq!(config.limits.todo, 7);
    assert_eq!(config.limits.in_progress, 3);
    assert_eq!(config.hints.learn_threshold, 10);
    assert_eq!(config.retention.done_discard_ttl_days, 12);
}

#[test]
fn load_rejects_non_positive_values() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("lazytask.toml"),
        "[limits]\ntodo = 0\nin_progress = 2\n\n[hints]\nlearn_threshold = 1\n",
    )
    .unwrap();

    let err = load_for_workspace_root(temp.path()).unwrap_err();
    assert!(err.to_string().contains("limits.todo must be >= 1"));

    fs::write(
        temp.path().join("lazytask.toml"),
        "[limits]\ntodo = 1\nin_progress = 2\n\n[hints]\nlearn_threshold = 0\n",
    )
    .unwrap();

    let err = load_for_workspace_root(temp.path()).unwrap_err();
    assert!(
        err.to_string()
            .contains("hints.learn_threshold must be >= 1")
    );

    fs::write(
        temp.path().join("lazytask.toml"),
        "[limits]\ntodo = 1\nin_progress = 2\n\n[hints]\nlearn_threshold = 1\n\n[retention]\ndone_discard_ttl_days = 0\n",
    )
    .unwrap();

    let err = load_for_workspace_root(temp.path()).unwrap_err();
    assert!(
        err.to_string()
            .contains("retention.done_discard_ttl_days must be >= 1")
    );
}

#[test]
fn workspace_root_rules_drive_config_path() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("nested/deep")).unwrap();

    let resolved = resolve_workspace_from(&root.join("nested/deep"), None).unwrap();
    let discovered_root = resolved.root;
    let config = load_for_workspace_root(discovered_root).unwrap();

    assert_eq!(config.config_path(), root.join("lazytask.toml"));
}

#[test]
fn ensure_default_file_creates_expected_schema() {
    let temp = TempDir::new().unwrap();
    let config = load_for_workspace_root(temp.path()).unwrap();

    ensure_default_file(&config).unwrap();

    let body = fs::read_to_string(temp.path().join("lazytask.toml")).unwrap();
    let expected = format!(
        "{}{}",
        schema::render_default_config_body(),
        prompts::render_prompts_section()
    );
    assert_eq!(body, expected);
}

#[test]
fn ensure_default_file_backfills_missing_keys_without_overwriting_existing_values() {
    let temp = TempDir::new().unwrap();
    let config = load_for_workspace_root(temp.path()).unwrap();
    fs::write(
        temp.path().join("lazytask.toml"),
        "[limits]\ntodo = 9 # custom todo limit\n",
    )
    .unwrap();

    ensure_default_file(&config).unwrap();

    let body = fs::read_to_string(temp.path().join("lazytask.toml")).unwrap();
    // Custom value preserved
    assert!(body.contains("todo = 9 # custom todo limit"));
    // Missing keys backfilled from schema
    for section in &schema::USER_CONFIG_SCHEMA {
        assert!(body.contains(&format!("[{}]", section.name)));
        for key in section.keys {
            assert!(
                body.contains(key.name),
                "missing key {} in section {}",
                key.name,
                section.name
            );
        }
    }
    // Prompts section backfilled
    assert!(body.contains("[prompts]"));
    assert!(body.contains("done_reflection"));
}

#[test]
fn ensure_default_file_is_idempotent_after_backfill() {
    let temp = TempDir::new().unwrap();
    let config = load_for_workspace_root(temp.path()).unwrap();
    fs::write(temp.path().join("lazytask.toml"), "[limits]\ntodo = 9\n").unwrap();

    ensure_default_file(&config).unwrap();
    let first = fs::read_to_string(temp.path().join("lazytask.toml")).unwrap();

    ensure_default_file(&config).unwrap();
    let second = fs::read_to_string(temp.path().join("lazytask.toml")).unwrap();

    assert_eq!(first, second);
}

#[test]
fn ensure_default_file_upgrade_overwrites_existing_values() {
    let temp = TempDir::new().unwrap();
    let config = load_for_workspace_root(temp.path()).unwrap();
    fs::write(
        temp.path().join("lazytask.toml"),
        "[limits]\ntodo = 9\nin_progress = 1\n\n[hints]\nlearn_threshold = 1\n",
    )
    .unwrap();

    ensure_default_file_with_upgrade(&config, true).unwrap();

    let body = fs::read_to_string(temp.path().join("lazytask.toml")).unwrap();
    let expected = format!(
        "{}{}",
        schema::render_default_config_body(),
        prompts::render_prompts_section()
    );
    assert_eq!(body, expected);
}

#[test]
fn load_reads_custom_done_reflection_prompt() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("lazytask.toml"),
        "[prompts]\ndone_reflection = 'My custom reflection prompt'\n",
    )
    .unwrap();

    let config = load_for_workspace_root(temp.path()).unwrap();

    assert_eq!(
        config.prompt_overrides.done_reflection.as_deref(),
        Some("My custom reflection prompt")
    );
}

#[test]
fn backfill_preserves_custom_done_reflection_prompt() {
    let temp = TempDir::new().unwrap();
    let config = load_for_workspace_root(temp.path()).unwrap();
    fs::write(
        temp.path().join("lazytask.toml"),
        "[limits]\ntodo = 5\n\n[prompts]\ndone_reflection = 'Custom prompt'\n",
    )
    .unwrap();

    ensure_default_file(&config).unwrap();

    let body = fs::read_to_string(temp.path().join("lazytask.toml")).unwrap();
    assert!(body.contains("Custom prompt"));
    assert!(body.contains("[prompts]"));
}

#[test]
fn resolve_workspace_from_env_override_absolute() {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join("custom");
    let resolved =
        resolve_workspace_from(temp.path(), Some(target.to_str().unwrap().to_string())).unwrap();
    assert_eq!(resolved.root, target);
    assert_eq!(resolved.source, WorkspaceSource::EnvOverride);
}

#[test]
fn resolve_workspace_from_env_override_relative() {
    let temp = TempDir::new().unwrap();
    let resolved = resolve_workspace_from(temp.path(), Some("relative-dir".to_string())).unwrap();
    assert_eq!(resolved.root, temp.path().join("relative-dir"));
    assert_eq!(resolved.source, WorkspaceSource::EnvOverride);
}

#[test]
fn resolve_workspace_from_git_root() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("nested/deep")).unwrap();

    let resolved = resolve_workspace_from(&root.join("nested/deep"), None).unwrap();
    assert_eq!(resolved.root, root);
    assert_eq!(resolved.source, WorkspaceSource::GitRoot);
}

#[test]
fn resolve_workspace_from_worktree() {
    let temp = TempDir::new().unwrap();
    let main_root = temp.path().join("main-repo");
    let main_git = main_root.join(".git");
    fs::create_dir_all(main_git.join("worktrees/feature-branch")).unwrap();

    let wt_root = temp.path().join("worktree-checkout");
    fs::create_dir_all(&wt_root).unwrap();
    let gitdir_path = main_git.join("worktrees/feature-branch");
    fs::write(
        wt_root.join(".git"),
        format!("gitdir: {}", gitdir_path.display()),
    )
    .unwrap();

    let resolved = resolve_workspace_from(&wt_root, None).unwrap();
    assert_eq!(resolved.root, main_root.canonicalize().unwrap());
    assert_eq!(resolved.source, WorkspaceSource::GitWorktree);
}

#[test]
fn resolve_workspace_from_no_git_falls_back_to_cwd() {
    let temp = TempDir::new().unwrap();
    let resolved = resolve_workspace_from(temp.path(), None).unwrap();
    assert_eq!(resolved.root, temp.path());
    assert_eq!(resolved.source, WorkspaceSource::Cwd);
}

#[test]
fn find_git_root_returns_none_when_no_git() {
    let temp = TempDir::new().unwrap();
    assert!(loading::find_git_root(temp.path()).is_none());
}

#[test]
fn find_git_root_returns_some_for_git_dir() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("sub")).unwrap();

    assert_eq!(loading::find_git_root(&root.join("sub")), Some(root));
}

#[test]
fn find_git_root_returns_main_root_for_worktree() {
    let temp = TempDir::new().unwrap();
    let main_root = temp.path().join("main");
    fs::create_dir_all(main_root.join(".git/worktrees/wt1")).unwrap();

    let wt = temp.path().join("wt");
    fs::create_dir_all(&wt).unwrap();
    fs::write(
        wt.join(".git"),
        format!("gitdir: {}", main_root.join(".git/worktrees/wt1").display()),
    )
    .unwrap();

    assert_eq!(
        loading::find_git_root(&wt),
        Some(main_root.canonicalize().unwrap())
    );
}

#[test]
fn resolve_worktree_malformed_git_file_returns_none() {
    let temp = TempDir::new().unwrap();
    let git_file = temp.path().join(".git");
    fs::write(&git_file, "not a valid gitdir line").unwrap();

    assert!(loading::resolve_worktree_main_root(&git_file).is_none());
}

#[test]
fn resolve_worktree_relative_gitdir() {
    let temp = TempDir::new().unwrap();
    let main_root = temp.path().join("main");
    fs::create_dir_all(main_root.join(".git/worktrees/feat")).unwrap();

    let wt = main_root.join("worktrees/feat-checkout");
    fs::create_dir_all(&wt).unwrap();
    fs::write(wt.join(".git"), "gitdir: ../../.git/worktrees/feat").unwrap();

    let result = loading::resolve_worktree_main_root(&wt.join(".git"));
    assert_eq!(result, Some(main_root.canonicalize().unwrap()));
}

#[test]
fn resolve_worktree_bare_repo_returns_none() {
    let temp = TempDir::new().unwrap();
    let bare = temp.path().join("repo.git");
    fs::create_dir_all(bare.join("worktrees/feat")).unwrap();

    let wt = temp.path().join("wt");
    fs::create_dir_all(&wt).unwrap();
    fs::write(
        wt.join(".git"),
        format!("gitdir: {}", bare.join("worktrees/feat").display()),
    )
    .unwrap();

    assert!(loading::resolve_worktree_main_root(&wt.join(".git")).is_none());
}
