use super::{LearningEntry, Storage, StorageError};
use crate::domain::{format_utc, parse_utc};
use chrono::{DateTime, Utc};
use std::fs;

impl Storage {
    /// Appends one completed-task learning entry to `LEARNINGS.md`.
    ///
    /// Entries are timestamped and written as a header plus markdown bullet lines.
    pub fn append_learning(
        &self,
        timestamp: DateTime<Utc>,
        task_title: &str,
        lines: &[String],
    ) -> Result<(), StorageError> {
        let mut content = String::new();
        content.push_str(&format!(
            "{} | {}\n",
            format_utc(timestamp),
            task_title.trim()
        ));
        for line in lines {
            content.push_str(&format!("- {}\n", line.trim()));
        }
        content.push('\n');

        let path = self.learnings_path();
        if path.exists() {
            let mut existing = fs::read_to_string(&path)?;
            if !existing.ends_with('\n') {
                existing.push('\n');
            }
            existing.push_str(&content);
            fs::write(path, existing)?;
        } else {
            fs::write(path, content)?;
        }

        Ok(())
    }

    /// Removes the learnings file when the learn queue has been consumed.
    pub fn clear_learnings(&self) -> Result<(), StorageError> {
        let path = self.learnings_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Counts non-empty lines in the learnings file for hint-threshold checks.
    pub fn learnings_line_count(&self) -> Result<usize, StorageError> {
        let path = self.learnings_path();
        if !path.exists() {
            return Ok(0);
        }
        let contents = fs::read_to_string(path)?;
        Ok(contents.lines().filter(|l| !l.trim().is_empty()).count())
    }

    /// Parses all persisted learning entries from disk.
    pub(crate) fn read_learning_entries(&self) -> Result<Vec<LearningEntry>, StorageError> {
        let path = self.learnings_path();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(path)?;
        parse_learning_entries(&contents)
    }
}

/// Parses serialized `LEARNINGS.md` content into structured entries.
pub(crate) fn parse_learning_entries(contents: &str) -> Result<Vec<LearningEntry>, StorageError> {
    let mut entries = Vec::new();
    let mut current: Option<LearningEntry> = None;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            continue;
        }

        if let Some(entry) = current.as_mut()
            && let Some(item) = trimmed.strip_prefix("- ")
        {
            entry.lines.push(item.trim().to_string());
            continue;
        }

        if let Some((ts, title)) = trimmed.split_once(" | ") {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }

            current = Some(LearningEntry {
                timestamp: parse_utc(ts).map_err(|err| StorageError::Parse(err.to_string()))?,
                task_title: title.trim().to_string(),
                lines: Vec::new(),
            });
        }
    }

    if let Some(entry) = current.take() {
        entries.push(entry);
    }

    Ok(entries)
}
