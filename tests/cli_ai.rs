use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

fn lt_command() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("lt"))
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}

fn init_temp() -> TempDir {
    let temp = TempDir::new().unwrap();
    let init = lt_command()
        .current_dir(temp.path())
        .arg("init")
        .output()
        .unwrap();
    assert!(init.status.success());
    temp
}

fn create_task_of_type(temp: &TempDir, title: &str, task_type: &str) {
    let output = lt_command()
        .current_dir(temp.path())
        .args([
            "create",
            "--title",
            title,
            "--type",
            task_type,
            "--details",
            "test desc",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
}

fn create_task(temp: &TempDir, title: &str) {
    create_task_of_type(temp, title, "task");
}

#[test]
fn no_subcommand_in_non_tty_returns_json_error() {
    let temp = TempDir::new().unwrap();
    let output = lt_command().current_dir(temp.path()).output().unwrap();
    let payload = parse_json(&output.stdout);

    assert!(!output.status.success());
    assert_eq!(payload["ok"], false);
    assert_eq!(payload["error"]["code"], "non_tty_requires_command");
}

#[test]
fn root_help_succeeds_and_prints_usage() {
    let temp = TempDir::new().unwrap();
    let output = lt_command()
        .current_dir(temp.path())
        .args(["--help"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Usage: lt"));
}

#[test]
fn subcommand_help_succeeds_and_prints_usage() {
    let temp = TempDir::new().unwrap();
    let output = lt_command()
        .current_dir(temp.path())
        .args(["create", "--help"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Usage: lt create"));
}

#[test]
fn ai_list_without_tasks_root_returns_machine_error() {
    let temp = TempDir::new().unwrap();
    let output = lt_command()
        .current_dir(temp.path())
        .args(["list"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(!output.status.success());
    assert_eq!(payload["ok"], false);
    assert_eq!(payload["error"]["code"], "tasks_root_missing");
}

#[test]
fn init_creates_lazytask_toml() {
    let temp = TempDir::new().unwrap();
    let output = lt_command()
        .current_dir(temp.path())
        .args(["init"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(temp.path().join("lazytask.toml").exists());
}

#[test]
fn init_from_nested_git_workspace_writes_config_at_workspace_root() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("repo");
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(root.join("nested/deep")).unwrap();

    let output = lt_command()
        .current_dir(root.join("nested/deep"))
        .args(["init"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(root.join("lazytask.toml").exists());
    assert!(root.join(".tasks").is_dir());
    assert!(!root.join("nested/deep/lazytask.toml").exists());
}

#[test]
fn ai_create_and_list_returns_success_envelope() {
    let temp = TempDir::new().unwrap();

    let init = lt_command()
        .current_dir(temp.path())
        .arg("init")
        .output()
        .unwrap();
    assert!(init.status.success());

    let create = lt_command()
        .current_dir(temp.path())
        .args([
            "create",
            "--title",
            "Ship rust",
            "--type",
            "task",
            "--details",
            "rewrite now",
        ])
        .output()
        .unwrap();

    let create_payload = parse_json(&create.stdout);
    assert!(create.status.success());
    assert_eq!(create_payload["ok"], true);
    assert_eq!(create_payload["data"]["task"]["status"], "todo");
    let create_task = create_payload["data"]["task"].as_object().unwrap();
    assert!(!create_task.contains_key("file_name"));
    assert!(!create_task.contains_key("created"));
    assert!(!create_task.contains_key("created_relative"));
    assert!(!create_task.contains_key("updated_relative"));
    assert!(!create_task["updated"].as_str().unwrap().contains('T'));

    let list = lt_command()
        .current_dir(temp.path())
        .args(["list"])
        .output()
        .unwrap();

    let list_payload = parse_json(&list.stdout);
    assert!(list.status.success());
    assert_eq!(list_payload["ok"], true);
    assert_eq!(list_payload["data"]["tasks"].as_array().unwrap().len(), 1);
    let listed_task = list_payload["data"]["tasks"][0].as_object().unwrap();
    assert!(!listed_task.contains_key("file_name"));
    assert!(!listed_task.contains_key("created"));
    assert!(!listed_task.contains_key("created_relative"));
    assert!(!listed_task.contains_key("updated_relative"));
    assert!(!listed_task["updated"].as_str().unwrap().contains('T'));
}

#[test]
fn ai_list_can_filter_by_type() {
    let temp = init_temp();
    create_task(&temp, "Write docs");
    create_task_of_type(&temp, "Fix auth bug", "bug");

    let by_type = lt_command()
        .current_dir(temp.path())
        .args(["list", "--type", "bug"])
        .output()
        .unwrap();
    let by_type_payload = parse_json(&by_type.stdout);
    assert!(by_type.status.success());
    assert_eq!(
        by_type_payload["data"]["tasks"].as_array().unwrap().len(),
        1
    );
    assert_eq!(by_type_payload["data"]["tasks"][0]["title"], "Fix auth bug");
}

#[test]
fn ai_create_requires_details_argument() {
    let temp = init_temp();

    let output = lt_command()
        .current_dir(temp.path())
        .args(["create", "--title", "Ship rust", "--type", "task"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(!output.status.success());
    assert_eq!(payload["ok"], false);
    assert_eq!(payload["error"]["code"], "invalid_arguments");
}

#[test]
fn ai_create_accepts_multiline_bullet_details() {
    let temp = init_temp();
    let details = "- something\n- something else";

    let output = lt_command()
        .current_dir(temp.path())
        .args([
            "create",
            "--title",
            "Bullet details",
            "--type",
            "task",
            "--details",
            details,
        ])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["data"]["task"]["details"], details);
}

#[test]
fn ai_create_normalizes_escaped_newlines_in_details() {
    let temp = init_temp();

    let output = lt_command()
        .current_dir(temp.path())
        .args([
            "create",
            "--title",
            "Escaped details",
            "--type",
            "task",
            "--details",
            "- one\\n- two",
        ])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["data"]["task"]["details"], "- one\n- two");
}

#[test]
fn ai_get_returns_task() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    let output = lt_command()
        .current_dir(temp.path())
        .args(["get", "Ship rust"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["data"]["tasks"][0]["title"], "Ship rust");
}

#[test]
fn ai_start_moves_to_in_progress() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    let output = lt_command()
        .current_dir(temp.path())
        .args(["start", "Ship rust"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    assert_eq!(payload["data"]["task"]["status"], "in-progress");
}

#[test]
fn ai_done_moves_to_done() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    lt_command()
        .current_dir(temp.path())
        .args(["start", "Ship rust"])
        .output()
        .unwrap();

    let output = lt_command()
        .current_dir(temp.path())
        .args(["done", "Ship rust", "--learning", "learned something"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    assert_eq!(payload["data"]["task"]["status"], "done");
}

#[test]
fn ai_discard_moves_to_discard_bucket_and_hides_from_list() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    let output = lt_command()
        .current_dir(temp.path())
        .args(["discard", "Ship rust"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    assert_eq!(payload["data"]["task"]["status"], "discard");
    assert!(temp.path().join(".tasks/discard/ship-rust.md").exists());

    let list = lt_command()
        .current_dir(temp.path())
        .args(["list"])
        .output()
        .unwrap();
    let list_payload = parse_json(&list.stdout);
    assert_eq!(list_payload["data"]["tasks"].as_array().unwrap().len(), 0);
}

#[test]
fn ai_done_hint_threshold_uses_config_value() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    std::fs::write(
        temp.path().join("lazytask.toml"),
        "[hints]\nlearn_threshold = 1\n",
    )
    .unwrap();

    let output = lt_command()
        .current_dir(temp.path())
        .args(["done", "Ship rust", "--learning", "learned something"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    let hint = payload["hint"].as_str().unwrap();
    assert!(hint.contains("Time to learn!"));
    assert!(hint.contains("learning session"));
    assert!(!hint.contains(".tasks/LEARNINGS.md has"));
}

#[test]
fn ai_done_normalizes_escaped_newlines_in_learning() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    lt_command()
        .current_dir(temp.path())
        .args(["start", "Ship rust"])
        .output()
        .unwrap();

    let done = lt_command()
        .current_dir(temp.path())
        .args(["done", "Ship rust", "--learning", "first\\nsecond"])
        .output()
        .unwrap();
    assert!(done.status.success());

    let learn = lt_command()
        .current_dir(temp.path())
        .args(["learn"])
        .output()
        .unwrap();
    let payload = parse_json(&learn.stdout);
    assert!(learn.status.success());

    let learnings = payload["data"]["learn"]["entries"][0]["learnings"]
        .as_str()
        .unwrap();
    assert_eq!(learnings, "first\nsecond");
}

#[test]
fn ai_list_defaults_to_active_tasks_only() {
    let temp = init_temp();
    create_task(&temp, "Todo task");
    create_task(&temp, "Done task");

    lt_command()
        .current_dir(temp.path())
        .args(["start", "Done task"])
        .output()
        .unwrap();
    lt_command()
        .current_dir(temp.path())
        .args(["done", "Done task", "--learning", "learned"])
        .output()
        .unwrap();

    let list = lt_command()
        .current_dir(temp.path())
        .args(["list"])
        .output()
        .unwrap();
    let payload = parse_json(&list.stdout);
    assert!(list.status.success());
    let tasks = payload["data"]["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "Todo task");
}

#[test]
fn ai_list_show_done_includes_completed_tasks() {
    let temp = init_temp();
    create_task(&temp, "Todo task");
    create_task(&temp, "Done task");

    lt_command()
        .current_dir(temp.path())
        .args(["start", "Done task"])
        .output()
        .unwrap();
    lt_command()
        .current_dir(temp.path())
        .args(["done", "Done task", "--learning", "learned"])
        .output()
        .unwrap();

    let list = lt_command()
        .current_dir(temp.path())
        .args(["list", "--show-done"])
        .output()
        .unwrap();
    let payload = parse_json(&list.stdout);
    assert!(list.status.success());
    let tasks = payload["data"]["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 2);
}

#[test]
fn ai_delete_removes_task() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    let output = lt_command()
        .current_dir(temp.path())
        .args(["delete", "Ship rust"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    assert_eq!(payload["data"]["task"]["title"], "Ship rust");

    let list = lt_command()
        .current_dir(temp.path())
        .args(["list"])
        .output()
        .unwrap();
    let list_payload = parse_json(&list.stdout);
    assert_eq!(list_payload["data"]["tasks"].as_array().unwrap().len(), 0);
}

#[test]
fn ai_learn_returns_entries() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    lt_command()
        .current_dir(temp.path())
        .args(["start", "Ship rust"])
        .output()
        .unwrap();

    lt_command()
        .current_dir(temp.path())
        .args(["done", "Ship rust", "--learning", "learned something"])
        .output()
        .unwrap();

    let output = lt_command()
        .current_dir(temp.path())
        .args(["learn"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    assert_eq!(
        payload["data"]["learn"]["entries"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(payload["data"]["learn"]["entries"][0]["title"], "Ship rust");
    let date = payload["data"]["learn"]["entries"][0]["date"]
        .as_str()
        .unwrap();
    assert_eq!(date.len(), 10);
    assert_eq!(date.chars().filter(|c| *c == '-').count(), 2);
    assert!(!date.contains('T'));
    assert_eq!(
        payload["data"]["learn"]["entries"][0]["learnings"],
        "learned something"
    );
    assert!(
        payload["data"]["learn"]["instructions"]
            .as_str()
            .unwrap()
            .contains("!!IMPORTANT: Follow this workflow to convert learnings to knowledge.")
    );
    assert!(payload.get("hint").is_none());

    // --finished clears learnings
    let finished = lt_command()
        .current_dir(temp.path())
        .args(["learn", "--finished"])
        .output()
        .unwrap();

    let finished_payload = parse_json(&finished.stdout);
    assert!(finished.status.success());
    assert_eq!(finished_payload["data"]["learn"]["cleared"], true);

    // learn after finished returns empty
    let empty = lt_command()
        .current_dir(temp.path())
        .args(["learn"])
        .output()
        .unwrap();

    let empty_payload = parse_json(&empty.stdout);
    assert!(empty.status.success());
    assert_eq!(
        empty_payload["data"]["learn"]["entries"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn ai_get_ambiguous_returns_error() {
    let temp = init_temp();
    create_task(&temp, "fix auth");
    create_task(&temp, "fix api");

    let output = lt_command()
        .current_dir(temp.path())
        .args(["get", "fix"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(!output.status.success());
    assert_eq!(payload["error"]["code"], "task_ambiguous");
}

#[test]
fn ai_get_not_found_returns_error() {
    let temp = init_temp();

    let output = lt_command()
        .current_dir(temp.path())
        .args(["get", "nonexistent"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(!output.status.success());
    assert_eq!(payload["error"]["code"], "task_not_found");
}

#[test]
fn ai_create_duplicate_returns_error() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    let output = lt_command()
        .current_dir(temp.path())
        .args([
            "create",
            "--title",
            "Ship rust",
            "--type",
            "task",
            "--details",
            "dup",
        ])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(!output.status.success());
    assert_eq!(payload["error"]["code"], "task_already_exists");
}
