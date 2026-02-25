use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lt::config::load_for_workspace_root;
use lt::domain::TaskType;
use lt::services::{CreateTaskInput, TaskService};
use lt::tui::actions::Action;
use lt::tui::app::App;
use std::path::Path;
use tempfile::TempDir;

fn runtime_for_path(path: &Path) -> (TaskService, usize) {
    let config = load_for_workspace_root(path).unwrap();
    let learn_threshold = config.hints.learn_threshold;
    (TaskService::new(config), learn_threshold)
}

#[test]
fn reducer_navigation_stays_in_bounds() {
    let temp = TempDir::new().unwrap();
    let (service, learn_threshold) = runtime_for_path(temp.path());
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Task one".to_string(),
            task_type: TaskType::Task,
            details: "a".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let mut app = App::new(service, learn_threshold);
    app.dispatch(Action::RefreshTasks);

    app.dispatch(Action::MoveSelectionUp);
    assert_eq!(app.state.selected_index, 0);

    app.dispatch(Action::MoveSelectionDown);
    assert_eq!(app.state.selected_index, 0);
}

#[test]
fn create_mode_cancel_returns_to_normal() {
    let temp = TempDir::new().unwrap();
    let (service, learn_threshold) = runtime_for_path(temp.path());
    service.init().unwrap();

    let mut app = App::new(service, learn_threshold);
    app.dispatch(Action::CreateTaskRequested);
    app.on_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert!(matches!(app.state.mode, lt::tui::app::Mode::Normal));
}

#[test]
fn create_submission_allows_empty_details() {
    let temp = TempDir::new().unwrap();
    let (service, learn_threshold) = runtime_for_path(temp.path());
    service.init().unwrap();

    let mut app = App::new(service, learn_threshold);
    app.dispatch(Action::CreateTaskSubmitted {
        title: "TUI without details".to_string(),
        task_type: TaskType::Task,
        details: String::new(),
    });

    assert_eq!(app.state.tasks.len(), 1);
    assert_eq!(app.state.tasks[0].title, "TUI without details");
    assert_eq!(app.state.tasks[0].details, "");
}

#[test]
fn delete_can_be_undone_and_logs_shortcut_hint() {
    let temp = TempDir::new().unwrap();
    let (service, learn_threshold) = runtime_for_path(temp.path());
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Undo me".to_string(),
            task_type: TaskType::Task,
            details: "details".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let mut app = App::new(service, learn_threshold);
    app.dispatch(Action::RefreshTasks);
    app.dispatch(Action::DeleteSelected);

    assert_eq!(app.state.tasks.len(), 0);
    assert_eq!(
        app.state.log_entries.back().unwrap().message,
        "task \"Undo me\" deleted (press u to undo)"
    );

    app.dispatch(Action::UndoDelete);
    assert_eq!(app.state.tasks.len(), 1);
    assert_eq!(app.state.tasks[0].title, "Undo me");
}

#[test]
fn startup_logs_learning_hint_when_line_count_exceeds_threshold() {
    let temp = TempDir::new().unwrap();
    std::fs::write(
        temp.path().join("lazytask.toml"),
        "[hints]\nlearn_threshold = 1\n",
    )
    .unwrap();
    let (service, learn_threshold) = runtime_for_path(temp.path());
    service.init().unwrap();

    let learnings = (0..=1)
        .map(|i| format!("- line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(temp.path().join(".tasks/LEARNINGS.md"), learnings).unwrap();

    let mut app = App::new(service, learn_threshold);
    app.dispatch(Action::CheckLearningHint);

    let latest_log = app.state.log_entries.back().unwrap();
    assert!(!latest_log.is_error);
    assert!(latest_log.message.contains("lt learn"));
    assert!(latest_log.message.contains("Ask your AI agent"));
}

#[test]
fn edit_mode_cancel_keeps_task_unchanged() {
    let temp = TempDir::new().unwrap();
    let (service, learn_threshold) = runtime_for_path(temp.path());
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Editable task".to_string(),
            task_type: TaskType::Task,
            details: "before".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let mut app = App::new(service, learn_threshold);
    app.dispatch(Action::RefreshTasks);
    app.on_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
    app.on_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    app.dispatch(Action::RefreshTasks);

    assert!(matches!(app.state.mode, lt::tui::app::Mode::Normal));
    assert_eq!(app.state.tasks.len(), 1);
    assert_eq!(app.state.tasks[0].title, "Editable task");
    assert_eq!(app.state.tasks[0].details, "before");
}

#[test]
fn edit_submission_overwrites_selected_task() {
    let temp = TempDir::new().unwrap();
    let (service, learn_threshold) = runtime_for_path(temp.path());
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Editable title".to_string(),
            task_type: TaskType::Task,
            details: "before".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();

    let mut app = App::new(service, learn_threshold);
    app.dispatch(Action::RefreshTasks);
    app.dispatch(Action::EditTaskSubmitted {
        file_name: "editable-title".to_string(),
        title: "Edited title".to_string(),
        task_type: TaskType::Bug,
        details: "after".to_string(),
    });

    assert_eq!(app.state.tasks.len(), 1);
    assert_eq!(app.state.tasks[0].title, "Edited title");
    assert_eq!(app.state.tasks[0].task_type, TaskType::Bug);
    assert_eq!(app.state.tasks[0].details, "after");
}

#[test]
fn discarded_task_cannot_be_edited_started_or_done() {
    let temp = TempDir::new().unwrap();
    let (service, learn_threshold) = runtime_for_path(temp.path());
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Discarded task".to_string(),
            task_type: TaskType::Task,
            details: "before".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();
    service.discard_task("discarded-task").unwrap();

    let mut app = App::new(service, learn_threshold);
    app.dispatch(Action::RefreshTasks);

    app.on_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
    assert!(matches!(app.state.mode, lt::tui::app::Mode::Normal));
    assert_eq!(
        app.state.log_entries.back().unwrap().message,
        "discarded tasks are terminal; delete instead"
    );

    app.dispatch(Action::StartSelected);
    assert_eq!(
        app.state.log_entries.back().unwrap().message,
        "start failed: discarded tasks are terminal"
    );

    app.dispatch(Action::DoneSelected);
    assert_eq!(
        app.state.log_entries.back().unwrap().message,
        "done failed: discarded tasks are terminal"
    );
}

#[test]
fn deleting_discarded_task_does_not_offer_undo() {
    let temp = TempDir::new().unwrap();
    let (service, learn_threshold) = runtime_for_path(temp.path());
    service.init().unwrap();

    service
        .create_task(CreateTaskInput {
            title: "Discard and delete".to_string(),
            task_type: TaskType::Task,
            details: "before".to_string(),
            start: false,
            require_details: true,
        })
        .unwrap();
    service.discard_task("discard-and-delete").unwrap();

    let mut app = App::new(service, learn_threshold);
    app.dispatch(Action::RefreshTasks);
    app.dispatch(Action::DeleteSelected);

    assert_eq!(app.state.tasks.len(), 0);
    assert!(app.state.last_deleted.is_none());
    assert_eq!(
        app.state.log_entries.back().unwrap().message,
        "discarded task \"Discard and delete\" deleted"
    );
}
