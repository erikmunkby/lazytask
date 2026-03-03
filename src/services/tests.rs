use super::*;
use crate::config::load_for_workspace_root;
use crate::domain::{TaskStatus, TaskType};
use std::fs;
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

    let task = service.done_task_without_learning("rewrite-cli").unwrap();
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
fn discard_task_with_note_normalizes_and_validates_note() {
    let temp = TempDir::new().unwrap();
    let service = service_for_temp(&temp);
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Document decision".to_string(),
            task_type: TaskType::Task,
            details: "details".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let discarded = service
        .discard_task_with_note("document-decision", "line one\\nline two")
        .unwrap();
    assert_eq!(discarded.status, TaskStatus::Discard);
    assert_eq!(
        discarded.discard_note.as_deref(),
        Some("line one\nline two")
    );

    let err = service
        .discard_task_with_note("document-decision", "   ")
        .unwrap_err();
    assert!(matches!(err, ServiceError::ValidationError(_)));
}

#[test]
fn create_ignores_discard_for_duplicate_check_but_done_still_blocks() {
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
    service
        .discard_task_with_note("duplicate-title", "wont do")
        .unwrap();

    let recreated = service
        .create_task(CreateTaskInput {
            title: "Duplicate title".to_string(),
            task_type: TaskType::Task,
            details: "second".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();
    assert_eq!(recreated.status, TaskStatus::Todo);

    service
        .done_task_without_learning("duplicate-title")
        .unwrap();
    let err = service
        .create_task(CreateTaskInput {
            title: "Duplicate title".to_string(),
            task_type: TaskType::Task,
            details: "third".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap_err();
    assert!(matches!(err, ServiceError::TaskAlreadyExists(_)));
}

#[test]
fn get_tasks_ignores_discarded_matches() {
    let temp = TempDir::new().unwrap();
    let service = service_for_temp(&temp);
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Query me".to_string(),
            task_type: TaskType::Task,
            details: "details".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();
    service
        .discard_task_with_note("query-me", "not now")
        .unwrap();

    let err = service.get_tasks(&["query me".to_string()]).unwrap_err();
    assert!(matches!(err, ServiceError::TaskNotFound(_)));
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

#[test]
fn cleanup_expired_terminal_tasks_deletes_done_and_discard_by_ttl() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("lazytask.toml"),
        "[retention]\ndone_discard_ttl_days = 7\n",
    )
    .unwrap();
    let service = service_for_temp(&temp);
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Old done".to_string(),
            task_type: TaskType::Task,
            details: "old".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();
    service.done_task_without_learning("old-done").unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Old discard".to_string(),
            task_type: TaskType::Task,
            details: "old".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();
    service.discard_task("old-discard").unwrap();

    let done_path = temp.path().join(".tasks/done/old-done.md");
    let discard_path = temp.path().join(".tasks/discard/old-discard.md");
    for path in [&done_path, &discard_path] {
        let content = fs::read_to_string(path).unwrap();
        let rewritten = content
            .lines()
            .map(|line| {
                if line.starts_with("updated: ") {
                    "updated: 2000-01-01T00:00:00Z".to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, format!("{rewritten}\n")).unwrap();
    }

    let deleted = service.cleanup_expired_terminal_tasks().unwrap();
    assert_eq!(deleted, 2);
    assert!(!done_path.exists());
    assert!(!discard_path.exists());
}

#[test]
fn add_learning_requires_done_status() {
    let temp = TempDir::new().unwrap();
    let service = service_for_temp(&temp);
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Not done yet".to_string(),
            task_type: TaskType::Task,
            details: "details".to_string(),
            start: true,
            require_details: true,
        })
        .unwrap();

    let err = service
        .add_learning_for_done_task("not-done-yet", "some learning")
        .unwrap_err();
    assert!(matches!(err, ServiceError::ValidationError(_)));

    service.done_task_without_learning("not-done-yet").unwrap();
    service
        .add_learning_for_done_task("not-done-yet", "some learning")
        .unwrap();
}
