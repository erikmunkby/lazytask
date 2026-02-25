use lt::config::load_for_workspace_root;
use lt::domain::{TaskStatus, TaskType};
use lt::services::{CreateTaskInput, ServiceError, TaskService};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn service_for_path(path: &Path) -> TaskService {
    let config = load_for_workspace_root(path).unwrap();
    TaskService::new(config)
}

#[test]
fn service_lifecycle_and_learn_finished() {
    let temp = TempDir::new().unwrap();
    let service = service_for_path(temp.path());
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Service task".to_string(),
            task_type: TaskType::Task,
            details: "implement service".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let started = service.start_task("service-task").unwrap();
    assert_eq!(started.status, TaskStatus::InProgress);

    let done = service
        .done_task_with_learning("service-task", "line one\nline two")
        .unwrap();
    assert_eq!(done.status, TaskStatus::Done);

    // learn returns all entries
    let first = service.learn().unwrap();
    assert_eq!(first.entries.len(), 1);
    assert!(!first.instructions.is_empty());
    assert_eq!(first.entries[0].title, "Service task");
    assert_eq!(first.entries[0].learnings, "line one\nline two");
    assert!(!first.entries[0].date.contains('T'));

    // learn again still returns entries until explicitly cleared
    let second = service.learn().unwrap();
    assert_eq!(second.entries.len(), 1);

    // learn_finished clears learnings
    service.learn_finished().unwrap();

    // learn after finished returns empty
    let third = service.learn().unwrap();
    assert!(third.entries.is_empty());
}

#[test]
fn service_list_can_filter_by_type_and_status() {
    let temp = TempDir::new().unwrap();
    let service = service_for_path(temp.path());
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Write docs".to_string(),
            task_type: TaskType::Task,
            details: "desc".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Fix login bug".to_string(),
            task_type: TaskType::Bug,
            details: "desc".to_string(),
            start: true,
            require_details: true,
        })
        .unwrap();

    service.done_task_without_learning("fix-login-bug").unwrap();

    let bugs = service.list_tasks(None, Some(TaskType::Bug)).unwrap();
    assert_eq!(bugs.len(), 1);
    assert_eq!(bugs[0].title, "Fix login bug");

    let done_bugs = service
        .list_tasks(Some(TaskStatus::Done), Some(TaskType::Bug))
        .unwrap();
    assert_eq!(done_bugs.len(), 1);
    assert_eq!(done_bugs[0].status, TaskStatus::Done);
}

#[test]
fn service_discard_moves_task_to_discard_status() {
    let temp = TempDir::new().unwrap();
    let service = service_for_path(temp.path());
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Duplicate task".to_string(),
            task_type: TaskType::Task,
            details: "desc".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let discarded = service.discard_task("duplicate-task").unwrap();
    assert_eq!(discarded.status, TaskStatus::Discard);

    let discarded_only = service
        .list_tasks(Some(TaskStatus::Discard), Some(TaskType::Task))
        .unwrap();
    assert_eq!(discarded_only.len(), 1);
    assert_eq!(discarded_only[0].title, "Duplicate task");
}

#[test]
fn service_discard_with_note_and_recreate_same_title() {
    let temp = TempDir::new().unwrap();
    let service = service_for_path(temp.path());
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Recreate me".to_string(),
            task_type: TaskType::Task,
            details: "desc".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let discarded = service
        .discard_task_with_note("recreate-me", "line one\\nline two")
        .unwrap();
    assert_eq!(
        discarded.discard_note.as_deref(),
        Some("line one\nline two")
    );

    let recreated = service
        .create_task(CreateTaskInput {
            title: "Recreate me".to_string(),
            task_type: TaskType::Task,
            details: "new desc".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    assert_eq!(recreated.status, TaskStatus::Todo);
}

#[test]
fn in_progress_limit_blocks_fourth_task() {
    let temp = TempDir::new().unwrap();
    let service = service_for_path(temp.path());
    service.init().unwrap();

    for i in 1..=3 {
        service
            .create_task(CreateTaskInput {
                title: format!("task {i}"),
                task_type: TaskType::Task,
                details: "desc".to_string(),
                start: true,
                require_details: true,
            })
            .unwrap();
    }

    let err = service
        .create_task(CreateTaskInput {
            title: "task 4".to_string(),
            task_type: TaskType::Task,
            details: "desc".to_string(),
            start: true,
            require_details: true,
        })
        .unwrap_err();

    assert!(matches!(err, ServiceError::StatusLimitReached(_)));
}

#[test]
fn learnings_line_count_empty_when_no_file() {
    let temp = TempDir::new().unwrap();
    let service = service_for_path(temp.path());
    service.init().unwrap();

    assert_eq!(service.learnings_line_count().unwrap(), 0);
}

#[test]
fn learnings_line_count_tracks_non_empty_lines() {
    let temp = TempDir::new().unwrap();
    let service = service_for_path(temp.path());
    service.init().unwrap();

    let content = "2026-02-21T14:00:00Z | task a\n- line 1\n- line 2\n\n";
    fs::write(temp.path().join(".tasks/LEARNINGS.md"), content).unwrap();

    assert_eq!(service.learnings_line_count().unwrap(), 3);
}

#[test]
fn learnings_line_count_above_threshold() {
    let temp = TempDir::new().unwrap();
    let service = service_for_path(temp.path());
    service.init().unwrap();

    // Generate >80 non-empty lines
    let mut content = String::new();
    for i in 0..30 {
        content.push_str(&format!("2026-02-21T14:00:00Z | task {i}\n"));
        content.push_str("- learning a\n");
        content.push_str("- learning b\n");
        content.push('\n');
    }
    fs::write(temp.path().join(".tasks/LEARNINGS.md"), &content).unwrap();

    let count = service.learnings_line_count().unwrap();
    assert!(count > 80, "expected >80 non-empty lines, got {count}");
}

#[test]
fn service_uses_limits_from_lazytask_toml() {
    let temp = TempDir::new().unwrap();
    std::fs::write(
        temp.path().join("lazytask.toml"),
        "[limits]\ntodo = 1\nin_progress = 3\n",
    )
    .unwrap();

    let service = service_for_path(temp.path());
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "todo 1".to_string(),
            task_type: TaskType::Task,
            details: "desc".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let err = service
        .create_task(CreateTaskInput {
            title: "todo 2".to_string(),
            task_type: TaskType::Task,
            details: "desc".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap_err();

    assert!(matches!(err, ServiceError::StatusLimitReached(_)));
}

#[test]
fn service_rejects_non_positive_limits_in_lazytask_toml() {
    let temp = TempDir::new().unwrap();
    std::fs::write(
        temp.path().join("lazytask.toml"),
        "[limits]\ntodo = 0\nin_progress = 1\n",
    )
    .unwrap();

    let err = load_for_workspace_root(temp.path()).unwrap_err();
    assert!(err.to_string().contains("limits.todo must be >= 1"));
}

#[test]
fn todo_limit_blocks_twenty_first_task() {
    let temp = TempDir::new().unwrap();
    let service = service_for_path(temp.path());
    service.init().unwrap();

    for i in 1..=20 {
        service
            .create_task(CreateTaskInput {
                title: format!("todo {i}"),
                task_type: TaskType::Task,
                details: "desc".to_string(),
                start: false,
                require_details: true,
            })
            .unwrap();
    }

    let err = service
        .create_task(CreateTaskInput {
            title: "todo 21".to_string(),
            task_type: TaskType::Task,
            details: "desc".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap_err();

    assert!(matches!(err, ServiceError::StatusLimitReached(_)));
}
