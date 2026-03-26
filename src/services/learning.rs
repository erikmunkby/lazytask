use super::errors::validation_error;
use super::{LearnEntry, LearnResult, ServiceError, TaskService};
use crate::config::markdown_for_key;
use crate::domain::{normalize_escaped_newlines, parse_learning_lines};
use chrono::Utc;

impl TaskService {
    /// Records a learning entry (not tied to any specific task).
    pub fn add_learning(&self, learning: &str) -> Result<(), ServiceError> {
        self.storage.require_layout()?;
        let normalized = normalize_escaped_newlines(learning);
        let learning_lines = parse_learning_lines(&normalized).map_err(validation_error)?;

        self.storage.append_learning(Utc::now(), &learning_lines)?;

        Ok(())
    }

    /// Returns pending learnings plus the prompt instructions for processing them.
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
                    date: entry.timestamp.date_naive().to_string(),
                    learnings: entry.lines.join("\n"),
                })
                .collect(),
            instructions: instructions.to_string(),
        })
    }

    /// Clears the learnings queue after a successful review cycle.
    pub fn learn_finished(&self) -> Result<(), ServiceError> {
        self.storage.require_layout()?;
        self.storage.clear_learnings()?;
        Ok(())
    }

    /// Returns the current learnings line count for threshold hinting.
    pub fn learnings_line_count(&self) -> Result<usize, ServiceError> {
        Ok(self.storage.learnings_line_count()?)
    }
}
