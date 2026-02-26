use super::{ServiceError, TaskService};
use crate::domain::Task;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::{Command, Stdio};

impl TaskService {
    /// Opens a task markdown file in the best available local editor command.
    pub fn open_task_in_editor(&self, task: &Task) -> Result<String, ServiceError> {
        self.storage.require_layout()?;
        let task_path = self.storage.task_path(task);
        open_path_in_editor(&task_path)
    }
}

/// Tries editor candidates in priority order until one succeeds.
fn open_path_in_editor(path: &Path) -> Result<String, ServiceError> {
    let candidates = build_candidates(&std::env::vars().collect());
    let mut failures = Vec::new();

    for candidate in candidates {
        match launch_candidate(&candidate, path) {
            Ok(()) => return Ok(candidate.label),
            Err(err) => failures.push(format!("{}: {err}", candidate.label)),
        }
    }

    Err(ServiceError::Io(format!(
        "unable to open task file {} ({})",
        path.display(),
        failures.join("; ")
    )))
}

/// Launches a single editor command and returns a user-readable failure message.
fn launch_candidate(candidate: &EditorCandidate, path: &Path) -> Result<(), String> {
    let mut command = Command::new(&candidate.program);
    command
        .args(&candidate.args)
        .arg(path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("exited with {status}")),
        Err(err) => Err(err.to_string()),
    }
}

/// Builds editor candidates from terminal context and editor environment variables.
fn build_candidates(env: &HashMap<String, String>) -> Vec<EditorCandidate> {
    let mut candidates = Vec::new();
    if in_cursor_terminal(env) {
        candidates.push(EditorCandidate::from_program("cursor"));
    }
    if in_vscode_terminal(env) {
        candidates.push(EditorCandidate::from_program("code"));
    }

    let editor_env = env
        .get("VISUAL")
        .filter(|value| !value.trim().is_empty())
        .or_else(|| env.get("EDITOR").filter(|value| !value.trim().is_empty()));
    if let Some(value) = editor_env
        && let Some(editor) = EditorCandidate::from_editor_env(value)
    {
        candidates.push(editor);
    }

    candidates.push(EditorCandidate::from_program("vi"));

    let mut seen = HashSet::new();
    candidates.retain(|candidate| seen.insert(candidate.dedupe_key()));
    candidates
}

/// Detects whether commands are running inside a Cursor-integrated terminal.
fn in_cursor_terminal(env: &HashMap<String, String>) -> bool {
    env.get("TERM_PROGRAM")
        .is_some_and(|value| value.eq_ignore_ascii_case("cursor"))
        || env.contains_key("CURSOR_TRACE_ID")
}

/// Detects whether commands are running inside a VS Code-integrated terminal.
fn in_vscode_terminal(env: &HashMap<String, String>) -> bool {
    env.get("TERM_PROGRAM")
        .is_some_and(|value| value.eq_ignore_ascii_case("vscode"))
        || env.contains_key("VSCODE_IPC_HOOK_CLI")
}

#[derive(Debug, Clone)]
struct EditorCandidate {
    label: String,
    program: String,
    args: Vec<String>,
}

impl EditorCandidate {
    /// Builds a candidate from a plain executable name.
    fn from_program(program: &str) -> Self {
        Self {
            label: program.to_string(),
            program: program.to_string(),
            args: Vec::new(),
        }
    }

    /// Parses `$VISUAL`/`$EDITOR` into executable plus argument list.
    fn from_editor_env(value: &str) -> Option<Self> {
        let mut parts = value.split_whitespace();
        let program = parts.next()?.trim();
        if program.is_empty() {
            return None;
        }
        Some(Self {
            label: value.trim().to_string(),
            program: program.to_string(),
            args: parts.map(ToString::to_string).collect(),
        })
    }

    /// Returns a stable key for deduplicating equivalent launch commands.
    fn dedupe_key(&self) -> String {
        if self.args.is_empty() {
            return self.program.clone();
        }
        format!("{} {}", self.program, self.args.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::build_candidates;
    use std::collections::HashMap;

    #[test]
    fn prefers_editor_cli_for_cursor_terminal() {
        let env = env_map(&[("TERM_PROGRAM", "cursor"), ("EDITOR", "nvim")]);
        let programs = build_candidates(&env)
            .into_iter()
            .map(|candidate| candidate.program)
            .collect::<Vec<_>>();
        assert_eq!(programs, vec!["cursor", "nvim", "vi"]);
    }

    #[test]
    fn visual_overrides_editor_variable() {
        let env = env_map(&[("VISUAL", "nano"), ("EDITOR", "nvim")]);
        let programs = build_candidates(&env)
            .into_iter()
            .map(|candidate| candidate.program)
            .collect::<Vec<_>>();
        assert_eq!(programs, vec!["nano", "vi"]);
    }

    #[test]
    fn includes_code_for_vscode_terminals() {
        let env = env_map(&[("VSCODE_IPC_HOOK_CLI", "/tmp/hook")]);
        let programs = build_candidates(&env)
            .into_iter()
            .map(|candidate| candidate.program)
            .collect::<Vec<_>>();
        assert_eq!(programs, vec!["code", "vi"]);
    }

    #[test]
    fn dedupes_fallback_vi() {
        let env = env_map(&[("EDITOR", "vi")]);
        let programs = build_candidates(&env)
            .into_iter()
            .map(|candidate| candidate.program)
            .collect::<Vec<_>>();
        assert_eq!(programs, vec!["vi"]);
    }

    fn env_map(values: &[(&str, &str)]) -> HashMap<String, String> {
        values
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }
}
