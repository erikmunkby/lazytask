use chrono::Utc;
use lazytask::config::load_for_workspace_root;
use lazytask::domain::{TaskStatus, TaskType};
use lazytask::storage::{Storage, StorageError};
use std::path::Path;
use tempfile::TempDir;

fn storage_for_path(path: &Path) -> Storage {
    let config = load_for_workspace_root(path).unwrap();
    Storage::from_app_config(&config)
}

#[test]
fn ensure_layout_creates_status_buckets() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_path(temp.path());

    storage.ensure_layout().unwrap();

    assert!(temp.path().join(".tasks").is_dir());
    assert!(temp.path().join(".tasks/todo").is_dir());
    assert!(temp.path().join(".tasks/in-progress").is_dir());
    assert!(temp.path().join(".tasks/done").is_dir());
    assert!(temp.path().join(".tasks/discard").is_dir());
}

#[test]
fn create_and_move_task_updates_bucket() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_path(temp.path());
    storage.ensure_layout().unwrap();

    let task = storage
        .create_task(
            "Storage task",
            TaskStatus::Todo,
            TaskType::Task,
            "desc",
            Utc::now(),
        )
        .unwrap();
    assert!(temp.path().join(".tasks/todo/storage-task.md").exists());

    let moved = storage
        .move_task(&task, TaskStatus::InProgress, Utc::now())
        .unwrap();

    assert_eq!(moved.status, TaskStatus::InProgress);
    assert!(!temp.path().join(".tasks/todo/storage-task.md").exists());
    assert!(
        temp.path()
            .join(".tasks/in-progress/storage-task.md")
            .exists()
    );
}

#[test]
fn clear_learnings_removes_file() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_path(temp.path());
    storage.ensure_layout().unwrap();

    // Write a learning entry
    storage
        .append_learning(Utc::now(), &["learned something".to_string()])
        .unwrap();
    assert!(temp.path().join(".tasks/LEARNINGS.md").exists());

    storage.clear_learnings().unwrap();

    assert!(!temp.path().join(".tasks/LEARNINGS.md").exists());
}

#[test]
fn require_layout_needs_all_status_buckets() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_path(temp.path());
    std::fs::create_dir_all(temp.path().join(".tasks")).unwrap();
    std::fs::create_dir_all(temp.path().join(".tasks/in-progress")).unwrap();
    std::fs::create_dir_all(temp.path().join(".tasks/done")).unwrap();

    let err = storage.require_layout().unwrap_err();
    assert!(matches!(err, StorageError::TasksRootMissing));
}

#[test]
fn require_layout_backfills_discard_bucket() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_path(temp.path());
    std::fs::create_dir_all(temp.path().join(".tasks/todo")).unwrap();
    std::fs::create_dir_all(temp.path().join(".tasks/in-progress")).unwrap();
    std::fs::create_dir_all(temp.path().join(".tasks/done")).unwrap();

    storage.require_layout().unwrap();
    assert!(temp.path().join(".tasks/discard").is_dir());
}
