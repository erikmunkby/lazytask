use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn load_uses_defaults_when_file_is_missing() {
    let temp = TempDir::new().unwrap();

    let config = load_for_workspace_root(temp.path()).unwrap();

    assert_eq!(config.limits.todo, 20);
    assert_eq!(config.limits.in_progress, 3);
    assert_eq!(config.hints.learn_threshold, 35);
    assert_eq!(config.retention.done_discard_ttl_days, 7);
    assert_eq!(config.storage_layout.tasks_dir, ".tasks");
    assert_eq!(config.storage_layout.learnings_file, "LEARNINGS.md");
    assert_eq!(
        config.prompts.learn_threshold_hint_key,
        "learn_threshold_hint"
    );
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

    let discovered_root = loading::find_workspace_root(&root.join("nested/deep"));
    let config = load_for_workspace_root(discovered_root).unwrap();

    assert_eq!(config.config_path(), root.join("lazytask.toml"));
}

#[test]
fn ensure_default_file_creates_expected_schema() {
    let temp = TempDir::new().unwrap();
    let config = load_for_workspace_root(temp.path()).unwrap();

    ensure_default_file(&config).unwrap();

    let body = fs::read_to_string(temp.path().join("lazytask.toml")).unwrap();
    assert!(body.contains("[limits]"));
    assert!(body.contains("todo = 20"));
    assert!(body.contains("# max todo tasks"));
    assert!(body.contains("in_progress = 3"));
    assert!(body.contains("# max in-progress tasks"));
    assert!(body.contains("[hints]"));
    assert!(body.contains("learn_threshold = 35"));
    assert!(body.contains("# show `lt learn` hint after this many LEARNINGS.md lines"));
    assert!(body.contains("[retention]"));
    assert!(body.contains("done_discard_ttl_days = 7"));
    assert!(body.contains("# auto-delete done/discard tasks older than this many days"));
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
    assert!(body.contains("todo = 9 # custom todo limit"));
    assert!(body.contains("in_progress = 3 # max in-progress tasks"));
    assert!(body.contains("[hints]"));
    assert!(body.contains(
        "learn_threshold = 35 # show `lt learn` hint after this many LEARNINGS.md lines"
    ));
    assert!(body.contains("[retention]"));
    assert!(body.contains(
        "done_discard_ttl_days = 7 # auto-delete done/discard tasks older than this many days"
    ));
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
    assert!(body.contains("todo = 20"));
    assert!(body.contains("in_progress = 3"));
    assert!(body.contains("learn_threshold = 35"));
    assert!(!body.contains("todo = 9"));
    assert!(!body.contains("learn_threshold = 1"));
}
