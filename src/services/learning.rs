use super::{LearnEntry, LearnResult, ServiceError, TaskService};
use crate::config::markdown_for_key;
use crate::domain::Task;

impl TaskService {
    pub fn learn(&self) -> Result<LearnResult, ServiceError> {
        self.storage.require_layout()?;
        let mut entries = self.storage.read_learning_entries()?;
        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        let instructions = markdown_for_key(self.config.prompts.learn_instructions_key)
            .ok_or_else(|| {
                ServiceError::ParseError(format!(
                    "unknown prompt key: {}",
                    self.config.prompts.learn_instructions_key
                ))
            })?;

        Ok(LearnResult {
            entries: entries
                .into_iter()
                .map(|entry| LearnEntry {
                    title: entry.task_title,
                    date: entry.timestamp.date_naive().to_string(),
                    learnings: entry.lines.join("\n"),
                })
                .collect(),
            instructions: instructions.to_string(),
        })
    }

    pub fn learn_finished(&self) -> Result<(), ServiceError> {
        self.storage.require_layout()?;
        self.storage.clear_learnings()?;
        Ok(())
    }

    pub fn learnings_line_count(&self) -> Result<usize, ServiceError> {
        Ok(self.storage.learnings_line_count()?)
    }

    pub fn read_task_content(&self, task: &Task) -> Result<String, ServiceError> {
        Ok(self.storage.read_task_content(task)?)
    }
}
