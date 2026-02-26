use super::*;
use crate::config::load_for_workspace_root;
use crate::domain::{TaskStatus, TaskType};
use chrono::{TimeZone, Utc};
use std::fs;
use tempfile::TempDir;

fn storage_for_temp(temp: &TempDir) -> Storage {
    let config = load_for_workspace_root(temp.path()).unwrap();
    Storage::from_app_config(&config)
}

#[test]
fn parses_learning_entries_file() {
    let entries = learning::parse_learning_entries(
        "2026-02-21T14:00:00Z | task a\n- line 1\n- line 2\n\n2026-02-21T15:00:00Z | task b\n- x\n- y\n",
    )
    .unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].task_title, "task a");
}

#[test]
fn round_trip_create_and_list_task() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_temp(&temp);
    storage.ensure_layout().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 2, 21, 15, 0, 0).unwrap();

    storage
        .create_task(
            "Ship rewrite",
            TaskStatus::Todo,
            TaskType::Task,
            "Do it",
            now,
        )
        .unwrap();

    let tasks = storage.list_tasks(None, None).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Ship rewrite");
}

#[test]
fn list_tasks_can_filter_by_type() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_temp(&temp);
    storage.ensure_layout().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 2, 21, 15, 0, 0).unwrap();

    storage
        .create_task(
            "Normal task",
            TaskStatus::Todo,
            TaskType::Task,
            "Do it",
            now,
        )
        .unwrap();
    storage
        .create_task("Fix auth", TaskStatus::Todo, TaskType::Bug, "Fix it", now)
        .unwrap();

    let bugs = storage.list_tasks(None, Some(TaskType::Bug)).unwrap();
    assert_eq!(bugs.len(), 1);
    assert_eq!(bugs[0].task_type, TaskType::Bug);
    assert_eq!(bugs[0].title, "Fix auth");
}

#[test]
fn round_trip_discard_note_in_markdown() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_temp(&temp);
    storage.ensure_layout().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 2, 21, 15, 0, 0).unwrap();

    let mut task = storage
        .create_task("Discard me", TaskStatus::Todo, TaskType::Task, "Do it", now)
        .unwrap();
    task.discard_note = Some("line one\nline two".to_string());
    storage.move_task(&task, TaskStatus::Discard, now).unwrap();

    let content = fs::read_to_string(temp.path().join(".tasks/discard/discard-me.md")).unwrap();
    assert!(content.contains("discard-note:"));
    assert!(content.contains("  line one"));
    assert!(content.contains("  line two"));

    let parsed = storage
        .list_tasks(Some(TaskStatus::Discard), None)
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(parsed.discard_note.as_deref(), Some("line one\nline two"));
}

#[test]
fn parse_task_without_discard_note_stays_compatible() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_temp(&temp);
    storage.ensure_layout().unwrap();

    let path = temp.path().join(".tasks/todo/no-note.md");
    fs::write(
        path,
        "# No note\nstatus: todo\ntype: task\ncreated: 2026-02-21T15:00:00Z\nupdated: 2026-02-21T15:00:00Z\ndetails:\n  text\n",
    )
    .unwrap();

    let task = storage
        .list_tasks(Some(TaskStatus::Todo), None)
        .unwrap()
        .pop()
        .unwrap();
    assert!(task.discard_note.is_none());
}

#[test]
fn init_prompt_uses_agents_file_by_default() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_temp(&temp);
    let prompts = storage.prompts;
    let prompt_markdown = crate::config::markdown_for_key(prompts.agent_init_key).unwrap();

    storage.ensure_agent_prompt_guidance().unwrap();

    let content = fs::read_to_string(temp.path().join(storage.layout.agents_file)).unwrap();
    let start = content.find(prompts.important_block_start).unwrap();
    let body_start = start + prompts.important_block_start.len();
    let body_end = content[body_start..]
        .find(prompts.important_block_end)
        .map(|idx| body_start + idx)
        .unwrap();
    let inserted_body = content[body_start..body_end].trim_matches('\n');
    assert_eq!(inserted_body, prompt_markdown.trim_matches('\n'));
}

#[test]
fn init_prompt_prefers_claude_when_agents_missing() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_temp(&temp);
    let prompts = storage.prompts;
    fs::write(temp.path().join(storage.layout.claude_file), "existing").unwrap();

    storage.ensure_agent_prompt_guidance().unwrap();

    let claude_content = fs::read_to_string(temp.path().join(storage.layout.claude_file)).unwrap();
    assert!(claude_content.contains(prompts.important_block_start));
    assert!(!temp.path().join(storage.layout.agents_file).exists());
}

#[test]
fn init_prompt_append_is_idempotent() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_temp(&temp);
    let prompts = storage.prompts;

    storage.ensure_agent_prompt_guidance().unwrap();
    storage.ensure_agent_prompt_guidance().unwrap();

    let content = fs::read_to_string(temp.path().join(storage.layout.agents_file)).unwrap();
    let count = content.matches(prompts.important_block_start).count();
    assert_eq!(count, 1);
}

#[test]
fn init_prompt_upgrade_rewrites_existing_lazytask_block() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_temp(&temp);
    let prompts = storage.prompts;
    let path = temp.path().join(storage.layout.agents_file);
    fs::write(
        &path,
        format!(
            "header\n{}\nold lazytask guidance\n{}\nfooter\n",
            prompts.important_block_start, prompts.important_block_end
        ),
    )
    .unwrap();

    storage
        .ensure_agent_prompt_guidance_with_upgrade(true)
        .unwrap();

    let prompt_markdown = crate::config::markdown_for_key(prompts.agent_init_key).unwrap();
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("header"));
    assert!(content.contains("footer"));
    assert!(content.contains(prompt_markdown.trim_matches('\n')));
    assert!(!content.contains("old lazytask guidance"));
}

#[test]
fn delete_terminal_tasks_updated_before_removes_only_expired_done_and_discard() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_temp(&temp);
    storage.ensure_layout().unwrap();

    let old = Utc.with_ymd_and_hms(2020, 1, 1, 12, 0, 0).unwrap();
    let recent = Utc.with_ymd_and_hms(2026, 2, 21, 12, 0, 0).unwrap();
    let cutoff = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();

    storage
        .create_task("done old", TaskStatus::Done, TaskType::Task, "old", old)
        .unwrap();
    storage
        .create_task(
            "discard old",
            TaskStatus::Discard,
            TaskType::Task,
            "old",
            old,
        )
        .unwrap();
    storage
        .create_task(
            "done recent",
            TaskStatus::Done,
            TaskType::Task,
            "recent",
            recent,
        )
        .unwrap();
    storage
        .create_task("todo old", TaskStatus::Todo, TaskType::Task, "old", old)
        .unwrap();
    storage
        .create_task(
            "in progress old",
            TaskStatus::InProgress,
            TaskType::Task,
            "old",
            old,
        )
        .unwrap();

    let deleted = storage
        .delete_terminal_tasks_updated_before(cutoff)
        .unwrap();
    assert_eq!(deleted, 2);

    assert!(!temp.path().join(".tasks/done/done-old.md").exists());
    assert!(!temp.path().join(".tasks/discard/discard-old.md").exists());
    assert!(temp.path().join(".tasks/done/done-recent.md").exists());
    assert!(temp.path().join(".tasks/todo/todo-old.md").exists());
    assert!(
        temp.path()
            .join(".tasks/in-progress/in-progress-old.md")
            .exists()
    );
}

#[test]
fn delete_terminal_tasks_updated_before_is_noop_when_tasks_root_missing() {
    let temp = TempDir::new().unwrap();
    let storage = storage_for_temp(&temp);
    let cutoff = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();

    let deleted = storage
        .delete_terminal_tasks_updated_before(cutoff)
        .unwrap();
    assert_eq!(deleted, 0);
}
