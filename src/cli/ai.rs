use super::command::{Commands, TaskData};
use crate::config::{AppConfig, markdown_for_key};
use crate::domain::{Task, TaskStatus, format_relative};
use crate::services::{CreateTaskInput, ServiceError, TaskService};
use serde_json::{Value, json};

pub(super) fn run_ai_command(
    service: &TaskService,
    config: &AppConfig,
    command: Commands,
) -> Result<(Value, Option<String>), ServiceError> {
    match command {
        Commands::Init => unreachable!("init is handled before AI dispatch"),
        Commands::List { task_type, show_done } => {
            let now = chrono::Utc::now();
            let statuses: Vec<TaskStatus> = if show_done {
                vec![TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done]
            } else {
                vec![TaskStatus::Todo, TaskStatus::InProgress]
            };
            let tasks = statuses
                .into_iter()
                .map(|s| service.list_tasks(Some(s), task_type))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .map(|task| {
                    json!({
                        "title": task.title,
                        "status": task.status,
                        "type": task.task_type,
                        "updated": format_relative(task.updated_at, now)
                    })
                })
                .collect::<Vec<_>>();
            Ok((json!({ "tasks": tasks }), None))
        }
        Commands::Get { query } => {
            let now = chrono::Utc::now();
            let tasks = service
                .get_tasks(&query)?
                .iter()
                .map(|task| to_task_data(task, now))
                .collect::<Vec<_>>();
            Ok((json!({ "tasks": tasks }), None))
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
            Ok((json!({ "task": to_task_data(&task, now) }), None))
        }
        Commands::Start { query } => {
            let now = chrono::Utc::now();
            let task = service.start_task(&query)?;
            Ok((json!({ "task": to_task_data(&task, now) }), None))
        }
        Commands::Done { query, learning } => {
            let now = chrono::Utc::now();
            let task = service.done_task_with_learning(&query, &learning)?;
            let hint = learnings_hint(service, config);
            Ok((json!({ "task": to_task_data(&task, now) }), hint))
        }
        Commands::Discard { query } => {
            let now = chrono::Utc::now();
            let task = service.discard_task(&query)?;
            Ok((json!({ "task": to_task_data(&task, now) }), None))
        }
        Commands::Delete { query } => {
            let now = chrono::Utc::now();
            let task = service.delete_task(&query)?;
            Ok((json!({ "task": to_task_data(&task, now) }), None))
        }
        Commands::Learn { finished } => {
            if finished {
                service.learn_finished()?;
                Ok((json!({ "learn": { "cleared": true } }), None))
            } else {
                let result = service.learn()?;
                Ok((json!({ "learn": result }), None))
            }
        }
    }
}

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

fn prompt_by_key(key: &str) -> Result<&'static str, ServiceError> {
    markdown_for_key(key)
        .ok_or_else(|| ServiceError::ParseError(format!("unknown prompt key: {key}")))
}

fn to_task_data(task: &Task, now: chrono::DateTime<chrono::Utc>) -> TaskData {
    TaskData {
        title: task.title.clone(),
        status: task.status.as_str().to_string(),
        task_type: task.task_type.as_str().to_string(),
        details: task.details.clone(),
        updated: format_relative(task.updated_at, now),
    }
}
