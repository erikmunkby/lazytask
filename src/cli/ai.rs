use super::command::{Commands, TaskData};
use crate::config::{AppConfig, markdown_for_key, resolve_done_reflection};
use crate::domain::{Task, TaskStatus, TaskType, format_relative};
use crate::services::{CreateTaskInput, ServiceError, TaskService};
use serde_json::{Value, json};

/// Runs one AI-facing command and returns a JSON-ready payload.
pub(super) fn run_ai_command(
    service: &TaskService,
    config: &AppConfig,
    command: Commands,
) -> Result<Value, ServiceError> {
    match command {
        Commands::Init { .. } => unreachable!("init is handled before AI dispatch"),
        Commands::List {
            task_type,
            show_done,
        } => {
            let now = chrono::Utc::now();
            let statuses: Vec<TaskStatus> = if show_done {
                vec![TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done]
            } else {
                vec![TaskStatus::Todo, TaskStatus::InProgress]
            };

            let mut result = json!({});
            for status in &statuses {
                let tasks = service.list_tasks(Some(*status), task_type)?;
                let mut by_type = json!({});
                for tt in &[TaskType::Task, TaskType::Bug] {
                    let items: Vec<Value> = tasks
                        .iter()
                        .filter(|t| t.task_type == *tt)
                        .map(|t| {
                            json!({
                                "title": t.title,
                                "updated": format_relative(t.updated_at, now)
                            })
                        })
                        .collect();
                    by_type[tt.as_str()] = json!(items);
                }
                result[status.as_str()] = by_type;
            }
            Ok(result)
        }
        Commands::Get { query } => {
            let now = chrono::Utc::now();
            let tasks: Vec<Value> = service
                .get_tasks(&query)?
                .iter()
                .map(|task| serde_json::to_value(to_task_data(task, now)).unwrap())
                .collect();
            Ok(json!(tasks))
        }
        Commands::Create {
            title,
            task_type,
            details,
            start,
        } => {
            let now = chrono::Utc::now();
            let task = service.create_task(CreateTaskInput {
                title,
                task_type,
                details,
                start,
                require_details: true,
            })?;
            Ok(serde_json::to_value(to_task_data(&task, now)).unwrap())
        }
        Commands::Start { query } => {
            let now = chrono::Utc::now();
            let task = service.start_task(&query)?;
            Ok(serde_json::to_value(to_task_data(&task, now)).unwrap())
        }
        Commands::Done { query } => {
            let now = chrono::Utc::now();
            let task = service.done_task_without_learning(&query)?;
            let mut data = to_task_data(&task, now);
            data.next_step = Some(
                resolve_done_reflection(config.prompt_overrides.done_reflection.as_deref())
                    .to_string(),
            );
            Ok(serde_json::to_value(data).unwrap())
        }
        Commands::Discard {
            query,
            discard_note,
        } => {
            let now = chrono::Utc::now();
            let task = service.discard_task_with_note(&query, &discard_note)?;
            Ok(serde_json::to_value(to_task_data(&task, now)).unwrap())
        }
        Commands::Delete { query } => {
            let now = chrono::Utc::now();
            let task = service.delete_task(&query)?;
            Ok(serde_json::to_value(to_task_data(&task, now)).unwrap())
        }
        Commands::Learn {
            query,
            learning,
            review,
            finished,
        } => match (query, learning, review, finished) {
            (Some(q), Some(l), false, false) => {
                let now = chrono::Utc::now();
                let task = service.add_learning_for_done_task(&q, &l)?;
                let mut data = to_task_data(&task, now);
                data.next_step = learnings_hint(service, config);
                Ok(serde_json::to_value(data).unwrap())
            }
            (None, None, true, false) => {
                let result = service.learn()?;
                Ok(serde_json::to_value(result).unwrap())
            }
            (None, None, false, true) => {
                service.learn_finished()?;
                Ok(json!({ "cleared": true }))
            }
            _ => Err(ServiceError::ValidationError(
                "usage: lt learn '<title>' --learning '<text>' | lt learn --review | lt learn --finished".to_string(),
            )),
        },
    }
}

/// Returns a learn-threshold hint when pending learnings exceed configured limits.
fn learnings_hint(service: &TaskService, config: &AppConfig) -> Option<String> {
    let count = service.learnings_line_count().unwrap_or(0);
    if count > config.hints.learn_threshold {
        prompt_by_key(config.prompts.learn_threshold_hint_key)
            .ok()
            .map(str::to_string)
    } else {
        None
    }
}

/// Resolves prompt markdown by key as a service-level parse error on misses.
fn prompt_by_key(key: &str) -> Result<&'static str, ServiceError> {
    markdown_for_key(key)
        .ok_or_else(|| ServiceError::ParseError(format!("unknown prompt key: {key}")))
}

/// Converts a domain task into the API response shape.
fn to_task_data(task: &Task, now: chrono::DateTime<chrono::Utc>) -> TaskData {
    TaskData {
        title: task.title.clone(),
        status: task.status.as_str().to_string(),
        task_type: task.task_type.as_str().to_string(),
        discard_note: task.discard_note.clone(),
        details: task.details.clone(),
        updated: format_relative(task.updated_at, now),
        next_step: None,
    }
}
