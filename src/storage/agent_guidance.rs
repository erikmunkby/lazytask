use super::{Storage, StorageError};
use crate::config::markdown_for_key;
use std::fs;

impl Storage {
    /// Ensures agent usage guidance is present in `AGENTS.md` or `CLAUDE.md`.
    ///
    /// If both files are missing, this creates `AGENTS.md` and appends the
    /// configured prompt block exactly once.
    pub fn ensure_agent_prompt_guidance(&self) -> Result<(), StorageError> {
        let agents_path = self.workspace_root.join(self.layout.agents_file);
        let claude_path = self.workspace_root.join(self.layout.claude_file);

        let target_path = if agents_path.exists() {
            agents_path
        } else if claude_path.exists() {
            claude_path
        } else {
            fs::write(&agents_path, "")?;
            agents_path
        };

        let content = if target_path.exists() {
            fs::read_to_string(&target_path)?
        } else {
            String::new()
        };

        if has_lazytask_important_block(&content, self.prompts) {
            return Ok(());
        }

        let prompt = markdown_for_key(self.prompts.agent_init_key).ok_or_else(|| {
            StorageError::Parse(format!(
                "unknown prompt key: {}",
                self.prompts.agent_init_key
            ))
        })?;
        let prefix = if !content.is_empty() && !content.ends_with('\n') {
            "\n"
        } else {
            ""
        };
        let updated = format!(
            "{content}{prefix}\n{}\n{}{}\n",
            self.prompts.important_block_start, prompt, self.prompts.important_block_end
        );
        fs::write(target_path, updated)?;
        Ok(())
    }
}

/// Detects whether an existing important block already contains lazytask guidance.
pub(crate) fn has_lazytask_important_block(
    content: &str,
    prompts: crate::config::PromptConfig,
) -> bool {
    let mut search_from = 0usize;
    while let Some(start_rel) = content[search_from..].find(prompts.important_block_start) {
        let start = search_from + start_rel + prompts.important_block_start.len();
        if let Some(end_rel) = content[start..].find(prompts.important_block_end) {
            let end = start + end_rel;
            let block = &content[start..end];
            let lowered = block.to_lowercase();
            if lowered.contains("lazytask") || block.contains("`lt`") {
                return true;
            }
            search_from = end + prompts.important_block_end.len();
        } else {
            break;
        }
    }
    false
}
