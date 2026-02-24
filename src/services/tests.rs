use super::*;
use crate::config::load_for_workspace_root;
use crate::domain::{TaskStatus, TaskType};
use tempfile::TempDir;

fn service_for_temp(temp: &TempDir) -> TaskService {
    let config = load_for_workspace_root(temp.path()).unwrap();
    TaskService::new(config)
}

#[test]
fn create_start_done_delete_flow() {
    let temp = TempDir::new().unwrap();
    let service = service_for_temp(&temp);
    service.init().unwrap();

    let task = service
        .create_task(CreateTaskInput {
            title: "Rewrite CLI".to_string(),
            task_type: TaskType::Task,
            details: "Implement command tree".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();
    assert_eq!(task.status, TaskStatus::Todo);

    let task = service.start_task("rewrite-cli").unwrap();
    assert_eq!(task.status, TaskStatus::InProgress);

    let task = service
        .done_task_with_learning("rewrite-cli", "line 1\nline 2")
        .unwrap();
    assert_eq!(task.status, TaskStatus::Done);

    let deleted = service.delete_task("rewrite-cli").unwrap();
    assert_eq!(deleted.title, "Rewrite CLI");
    assert!(service.list_tasks(None, None).unwrap().is_empty());
}

#[test]
fn restore_deleted_task() {
    let temp = TempDir::new().unwrap();
    let service = service_for_temp(&temp);
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Bring me back".to_string(),
            task_type: TaskType::Task,
            details: "details".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let deleted = service.delete_task("bring-me-back").unwrap();
    let restored = service.restore_task(&deleted).unwrap();

    assert_eq!(restored.title, "Bring me back");
    assert_eq!(restored.status, TaskStatus::Todo);
    assert_eq!(service.list_tasks(None, None).unwrap().len(), 1);
}

#[test]
fn restore_fails_when_file_name_already_exists() {
    let temp = TempDir::new().unwrap();
    let service = service_for_temp(&temp);
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Duplicate title".to_string(),
            task_type: TaskType::Task,
            details: "first".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let deleted = service.delete_task("duplicate-title").unwrap();
    service
        .create_task(CreateTaskInput {
            title: "Duplicate title".to_string(),
            task_type: TaskType::Task,
            details: "second".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let err = service.restore_task(&deleted).unwrap_err();
    assert!(matches!(err, ServiceError::TaskAlreadyExists(_)));
}

#[test]
fn create_task_normalizes_escaped_newlines_in_details() {
    let temp = TempDir::new().unwrap();
    let service = service_for_temp(&temp);
    service.init().unwrap();

    let task = service
        .create_task(CreateTaskInput {
            title: "Escaped details".to_string(),
            task_type: TaskType::Task,
            details: "line one\\nline two".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    assert_eq!(task.details, "line one\nline two");
}

#[test]
fn discard_task_moves_task_to_discard() {
    let temp = TempDir::new().unwrap();
    let service = service_for_temp(&temp);
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Old duplicate".to_string(),
            task_type: TaskType::Task,
            details: "details".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let discarded = service.discard_task("old-duplicate").unwrap();
    assert_eq!(discarded.status, TaskStatus::Discard);
}

#[test]
fn edit_task_overwrites_selected_task() {
    let temp = TempDir::new().unwrap();
    let service = service_for_temp(&temp);
    service.init().unwrap();

    let created = service
        .create_task(CreateTaskInput {
            title: "Edit me".to_string(),
            task_type: TaskType::Task,
            details: "before".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let edited = service
        .edit_task(
            "edit-me",
            "Edited title".to_string(),
            TaskType::Bug,
            "after".to_string(),
        )
        .unwrap();

    assert_eq!(edited.file_name, created.file_name);
    assert_eq!(edited.status, created.status);
    assert_eq!(edited.title, "Edited title");
    assert_eq!(edited.task_type, TaskType::Bug);
    assert_eq!(edited.details, "after");
    assert!(edited.updated_at >= created.updated_at);
}

#[test]
fn edit_task_normalizes_escaped_newlines_in_details() {
    let temp = TempDir::new().unwrap();
    let service = service_for_temp(&temp);
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Edit escaped details".to_string(),
            task_type: TaskType::Task,
            details: "before".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let edited = service
        .edit_task(
            "edit-escaped-details",
            "Edit escaped details".to_string(),
            TaskType::Task,
            "line one\\nline two".to_string(),
        )
        .unwrap();

    assert_eq!(edited.details, "line one\nline two");
}
