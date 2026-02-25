use super::ServiceError;
use crate::domain::{Task, TaskStatus};

pub(super) fn resolve_query(tasks: &[Task], query: &str) -> Result<Task, ServiceError> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Err(ServiceError::ValidationError(
            "task query cannot be empty".to_string(),
        ));
    }

    let mut exact = Vec::new();
    let mut fuzzy = Vec::new();

    for task in tasks {
        if task.status == TaskStatus::Discard {
            continue;
        }
        let title = task.title.to_lowercase();
        let file_name = task.file_name.to_lowercase();
        if title == needle || file_name == needle {
            exact.push(task.clone());
        } else if title.contains(&needle) || file_name.contains(&needle) {
            fuzzy.push(task.clone());
        }
    }

    let matches = if exact.is_empty() { fuzzy } else { exact };

    match matches.len() {
        0 => Err(ServiceError::TaskNotFound(query.to_string())),
        1 => Ok(matches[0].clone()),
        _ => Err(ServiceError::TaskAmbiguous {
            query: query.to_string(),
            matches: matches.into_iter().map(|task| task.title).collect(),
        }),
    }
}
