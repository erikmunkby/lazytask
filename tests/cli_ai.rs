use assert_cmd::Command;
use serde_json::Value;
use std::fs;
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

fn write_task_file(
    temp: &TempDir,
    bucket: &str,
    file_name: &str,
    title: &str,
    status: &str,
    updated: &str,
) {
    let path = temp.path().join(format!(".tasks/{bucket}/{file_name}.md"));
    fs::write(
        path,
        format!(
            "# {title}\nstatus: {status}\ntype: task\ncreated: 2000-01-01T00:00:00Z\nupdated: {updated}\ndetails:\n  seeded\n"
        ),
    )
    .unwrap();
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
fn init_backfills_missing_keys_without_overwriting_existing_values() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("lazytask.toml"),
        "[limits]\ntodo = 9 # custom todo limit\n",
    )
    .unwrap();

    let output = lt_command()
        .current_dir(temp.path())
        .args(["init"])
        .output()
        .unwrap();

    assert!(output.status.success());

    let body = fs::read_to_string(temp.path().join("lazytask.toml")).unwrap();
    assert!(body.contains("todo = 9 # custom todo limit"));
    assert!(body.contains("in_progress ="));
    assert!(body.contains("[hints]"));
    assert!(body.contains("learn_threshold ="));
    assert!(body.contains("[retention]"));
    assert!(body.contains("done_discard_ttl_days ="));
}

#[test]
fn init_upgrade_overwrites_config_and_guidance_without_touching_tasks() {
    let temp = TempDir::new().unwrap();
    let init = lt_command()
        .current_dir(temp.path())
        .args(["init"])
        .output()
        .unwrap();
    assert!(init.status.success());

    let keep_path = temp.path().join(".tasks/todo/keep-me.md");
    let keep_body = "# Keep me\nstatus: todo\ntype: task\ncreated: 2026-01-01T00:00:00Z\nupdated: 2026-01-01T00:00:00Z\ndetails:\n  keep\n";
    fs::write(&keep_path, keep_body).unwrap();
    fs::write(temp.path().join("lazytask.toml"), "[limits]\ntodo = 99\n").unwrap();
    fs::write(
        temp.path().join("AGENTS.md"),
        "before\n<EXTREMELY_IMPORTANT>\nold lazytask guidance\n</EXTREMELY_IMPORTANT>\nafter\n",
    )
    .unwrap();

    let upgrade = lt_command()
        .current_dir(temp.path())
        .args(["init", "--upgrade"])
        .output()
        .unwrap();
    assert!(upgrade.status.success());

    let config = fs::read_to_string(temp.path().join("lazytask.toml")).unwrap();
    assert!(config.contains("todo = 20"));
    assert!(!config.contains("todo = 99"));

    let agents = fs::read_to_string(temp.path().join("AGENTS.md")).unwrap();
    assert!(
        agents.contains("ALWAYS use lazytask (`lt`) for task and bug tracking in this project.")
    );
    assert!(!agents.contains("old lazytask guidance"));

    assert!(keep_path.exists());
    assert_eq!(fs::read_to_string(keep_path).unwrap(), keep_body);
}

#[test]
fn runtime_cleanup_removes_expired_done_and_discard_before_list() {
    let temp = init_temp();
    create_task(&temp, "Recent done");
    lt_command()
        .current_dir(temp.path())
        .args(["done", "Recent done"])
        .output()
        .unwrap();

    write_task_file(
        &temp,
        "done",
        "expired-done",
        "Expired done",
        "done",
        "2000-01-01T00:00:00Z",
    );
    write_task_file(
        &temp,
        "discard",
        "expired-discard",
        "Expired discard",
        "discard",
        "2000-01-01T00:00:00Z",
    );

    let output = lt_command()
        .current_dir(temp.path())
        .args(["list", "--show-done"])
        .output()
        .unwrap();
    let payload = parse_json(&output.stdout);

    assert!(output.status.success());
    let tasks = payload["done"]["task"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "Recent done");
    assert!(!temp.path().join(".tasks/done/expired-done.md").exists());
    assert!(
        !temp
            .path()
            .join(".tasks/discard/expired-discard.md")
            .exists()
    );
}

#[test]
fn ai_create_and_list_returns_flat_output() {
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
    assert_eq!(create_payload["status"], "todo");
    let create_task = create_payload.as_object().unwrap();
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
    assert_eq!(list_payload["todo"]["task"].as_array().unwrap().len(), 1);
    let listed_task = list_payload["todo"]["task"][0].as_object().unwrap();
    assert!(!listed_task.contains_key("file_name"));
    assert!(!listed_task.contains_key("created"));
    assert!(!listed_task.contains_key("created_relative"));
    assert!(!listed_task.contains_key("updated_relative"));
    assert!(!listed_task["updated"].as_str().unwrap().contains('T'));
}

#[test]
fn ai_list_groups_by_status_and_type() {
    let temp = init_temp();
    create_task(&temp, "Write docs");
    create_task_of_type(&temp, "Fix auth bug", "bug");

    let list = lt_command()
        .current_dir(temp.path())
        .args(["list"])
        .output()
        .unwrap();
    let payload = parse_json(&list.stdout);
    assert!(list.status.success());

    // Both statuses present
    assert!(payload.get("todo").is_some());
    assert!(payload.get("in-progress").is_some());
    // No done without --show-done
    assert!(payload.get("done").is_none());

    // Tasks grouped by type
    assert_eq!(payload["todo"]["task"].as_array().unwrap().len(), 1);
    assert_eq!(payload["todo"]["bug"].as_array().unwrap().len(), 1);
    assert_eq!(payload["todo"]["task"][0]["title"], "Write docs");
    assert_eq!(payload["todo"]["bug"][0]["title"], "Fix auth bug");
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
    let payload = parse_json(&by_type.stdout);
    assert!(by_type.status.success());
    assert_eq!(payload["todo"]["bug"].as_array().unwrap().len(), 1);
    assert_eq!(payload["todo"]["bug"][0]["title"], "Fix auth bug");
    assert_eq!(payload["todo"]["task"].as_array().unwrap().len(), 0);
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
    assert_eq!(payload["details"], details);
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
    assert_eq!(payload["details"], "- one\n- two");
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
    assert_eq!(payload[0]["title"], "Ship rust");
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
    assert_eq!(payload["status"], "in-progress");
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
        .args(["done", "Ship rust"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    assert_eq!(payload["status"], "done");
    let next_step = payload["next_step"].as_str().unwrap();
    assert!(!next_step.is_empty());
    assert!(next_step.contains("reflect"));
}

#[test]
fn ai_discard_moves_to_discard_bucket_and_hides_from_list() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    let output = lt_command()
        .current_dir(temp.path())
        .args(["discard", "Ship rust", "--discard-note", "out of scope"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    assert_eq!(payload["status"], "discard");
    assert_eq!(payload["discard_note"], "out of scope");
    assert!(temp.path().join(".tasks/discard/ship-rust.md").exists());

    let list = lt_command()
        .current_dir(temp.path())
        .args(["list"])
        .output()
        .unwrap();
    let list_payload = parse_json(&list.stdout);
    assert_eq!(list_payload["todo"]["task"].as_array().unwrap().len(), 0);
}

#[test]
fn ai_discard_requires_note() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    let output = lt_command()
        .current_dir(temp.path())
        .args(["discard", "Ship rust"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(!output.status.success());
    assert_eq!(payload["error"]["code"], "invalid_arguments");
}

#[test]
fn ai_discard_create_same_title_succeeds() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    let discard = lt_command()
        .current_dir(temp.path())
        .args(["discard", "Ship rust", "--discard-note", "not needed"])
        .output()
        .unwrap();
    assert!(discard.status.success());

    let recreate = lt_command()
        .current_dir(temp.path())
        .args([
            "create",
            "--title",
            "Ship rust",
            "--type",
            "task",
            "--details",
            "new work",
        ])
        .output()
        .unwrap();
    let recreate_payload = parse_json(&recreate.stdout);

    assert!(recreate.status.success());
    assert_eq!(recreate_payload["status"], "todo");
}

#[test]
fn ai_queries_ignore_discarded_duplicate() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    lt_command()
        .current_dir(temp.path())
        .args(["discard", "Ship rust", "--discard-note", "done elsewhere"])
        .output()
        .unwrap();

    lt_command()
        .current_dir(temp.path())
        .args([
            "create",
            "--title",
            "Ship rust",
            "--type",
            "task",
            "--details",
            "fresh",
        ])
        .output()
        .unwrap();

    let get = lt_command()
        .current_dir(temp.path())
        .args(["get", "Ship rust"])
        .output()
        .unwrap();
    let get_payload = parse_json(&get.stdout);
    assert!(get.status.success());
    assert_eq!(get_payload[0]["status"], "todo");

    let start = lt_command()
        .current_dir(temp.path())
        .args(["start", "Ship rust"])
        .output()
        .unwrap();
    let start_payload = parse_json(&start.stdout);
    assert!(start.status.success());
    assert_eq!(start_payload["status"], "in-progress");

    let done = lt_command()
        .current_dir(temp.path())
        .args(["done", "Ship rust"])
        .output()
        .unwrap();
    let done_payload = parse_json(&done.stdout);
    assert!(done.status.success());
    assert_eq!(done_payload["status"], "done");
}

#[test]
fn ai_querying_only_discarded_task_returns_not_found() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    lt_command()
        .current_dir(temp.path())
        .args(["discard", "Ship rust", "--discard-note", "obsolete"])
        .output()
        .unwrap();

    let get = lt_command()
        .current_dir(temp.path())
        .args(["get", "Ship rust"])
        .output()
        .unwrap();
    let get_payload = parse_json(&get.stdout);
    assert!(!get.status.success());
    assert_eq!(get_payload["error"]["code"], "task_not_found");
}

#[test]
fn ai_discard_note_validates_length_and_allows_multiline() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    let empty = lt_command()
        .current_dir(temp.path())
        .args(["discard", "Ship rust", "--discard-note", "   "])
        .output()
        .unwrap();
    let empty_payload = parse_json(&empty.stdout);
    assert!(!empty.status.success());
    assert_eq!(empty_payload["error"]["code"], "validation_error");

    let too_long_note = "x".repeat(121);
    let too_long = lt_command()
        .current_dir(temp.path())
        .args(["discard", "Ship rust", "--discard-note", &too_long_note])
        .output()
        .unwrap();
    let too_long_payload = parse_json(&too_long.stdout);
    assert!(!too_long.status.success());
    assert_eq!(too_long_payload["error"]["code"], "validation_error");

    let ok = lt_command()
        .current_dir(temp.path())
        .args([
            "discard",
            "Ship rust",
            "--discard-note",
            "line one\\nline two",
        ])
        .output()
        .unwrap();
    let ok_payload = parse_json(&ok.stdout);
    assert!(ok.status.success());
    assert_eq!(ok_payload["discard_note"], "line one\nline two");
}

#[test]
fn ai_learn_hint_threshold_uses_config_value() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    std::fs::write(
        temp.path().join("lazytask.toml"),
        "[hints]\nlearn_threshold = 1\n",
    )
    .unwrap();

    lt_command()
        .current_dir(temp.path())
        .args(["done", "Ship rust"])
        .output()
        .unwrap();

    let output = lt_command()
        .current_dir(temp.path())
        .args(["learn", "Ship rust", "--learning", "learned something"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    let next_step = payload["next_step"].as_str().unwrap();
    assert!(next_step.contains("Time to learn!"));
    assert!(next_step.contains("learning review"));
}

#[test]
fn ai_learn_normalizes_escaped_newlines_in_learning() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    lt_command()
        .current_dir(temp.path())
        .args(["start", "Ship rust"])
        .output()
        .unwrap();

    lt_command()
        .current_dir(temp.path())
        .args(["done", "Ship rust"])
        .output()
        .unwrap();

    let learn_add = lt_command()
        .current_dir(temp.path())
        .args(["learn", "Ship rust", "--learning", "first\\nsecond"])
        .output()
        .unwrap();
    assert!(learn_add.status.success());

    let learn_review = lt_command()
        .current_dir(temp.path())
        .args(["learn", "--review"])
        .output()
        .unwrap();
    let payload = parse_json(&learn_review.stdout);
    assert!(learn_review.status.success());

    let learnings = payload["entries"][0]["learnings"].as_str().unwrap();
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
        .args(["done", "Done task"])
        .output()
        .unwrap();

    let list = lt_command()
        .current_dir(temp.path())
        .args(["list"])
        .output()
        .unwrap();
    let payload = parse_json(&list.stdout);
    assert!(list.status.success());
    let tasks = payload["todo"]["task"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "Todo task");
    // No done bucket without --show-done
    assert!(payload.get("done").is_none());
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
        .args(["done", "Done task"])
        .output()
        .unwrap();

    let list = lt_command()
        .current_dir(temp.path())
        .args(["list", "--show-done"])
        .output()
        .unwrap();
    let payload = parse_json(&list.stdout);
    assert!(list.status.success());
    // todo has 1 task, done has 1 task
    assert_eq!(payload["todo"]["task"].as_array().unwrap().len(), 1);
    assert_eq!(payload["done"]["task"].as_array().unwrap().len(), 1);
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
    assert_eq!(payload["title"], "Ship rust");

    let list = lt_command()
        .current_dir(temp.path())
        .args(["list"])
        .output()
        .unwrap();
    let list_payload = parse_json(&list.stdout);
    assert_eq!(list_payload["todo"]["task"].as_array().unwrap().len(), 0);
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
        .args(["done", "Ship rust"])
        .output()
        .unwrap();

    // add learning via learn command
    let learn_add = lt_command()
        .current_dir(temp.path())
        .args(["learn", "Ship rust", "--learning", "learned something"])
        .output()
        .unwrap();
    assert!(learn_add.status.success());

    // review learnings
    let output = lt_command()
        .current_dir(temp.path())
        .args(["learn", "--review"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(output.status.success());
    assert_eq!(payload["entries"].as_array().unwrap().len(), 1);
    assert_eq!(payload["entries"][0]["title"], "Ship rust");
    let date = payload["entries"][0]["date"].as_str().unwrap();
    assert_eq!(date.len(), 10);
    assert_eq!(date.chars().filter(|c| *c == '-').count(), 2);
    assert!(!date.contains('T'));
    assert_eq!(payload["entries"][0]["learnings"], "learned something");
    let instructions = payload["instructions"].as_str().unwrap();
    assert!(!instructions.is_empty());

    // --finished clears learnings
    let finished = lt_command()
        .current_dir(temp.path())
        .args(["learn", "--finished"])
        .output()
        .unwrap();

    let finished_payload = parse_json(&finished.stdout);
    assert!(finished.status.success());
    assert_eq!(finished_payload["cleared"], true);

    // learn --review after finished returns empty
    let empty = lt_command()
        .current_dir(temp.path())
        .args(["learn", "--review"])
        .output()
        .unwrap();

    let empty_payload = parse_json(&empty.stdout);
    assert!(empty.status.success());
    assert_eq!(empty_payload["entries"].as_array().unwrap().len(), 0);
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

#[test]
fn ai_learn_validates_task_must_be_done() {
    let temp = init_temp();
    create_task(&temp, "Ship rust");

    lt_command()
        .current_dir(temp.path())
        .args(["start", "Ship rust"])
        .output()
        .unwrap();

    let output = lt_command()
        .current_dir(temp.path())
        .args(["learn", "Ship rust", "--learning", "some learning"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(!output.status.success());
    assert_eq!(payload["error"]["code"], "validation_error");
}

#[test]
fn ai_bare_learn_returns_validation_error() {
    let temp = init_temp();

    let output = lt_command()
        .current_dir(temp.path())
        .args(["learn"])
        .output()
        .unwrap();

    let payload = parse_json(&output.stdout);
    assert!(!output.status.success());
    assert_eq!(payload["error"]["code"], "validation_error");
}
