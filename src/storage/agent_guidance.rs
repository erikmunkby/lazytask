use super::{Storage, StorageError};
use crate::config::markdown_for_key;
use std::fs;

impl Storage {
    /// Ensures agent usage guidance is present in `AGENTS.md` or `CLAUDE.md`.
    ///
    /// If both files are missing, this creates `AGENTS.md` and appends the
    /// configured prompt block exactly once.
    pub fn ensure_agent_prompt_guidance(&self) -> Result<(), StorageError> {
        self.ensure_agent_prompt_guidance_with_upgrade(false)
    }

    /// Ensures agent guidance exists and optionally refreshes the lazytask block.
    pub fn ensure_agent_prompt_guidance_with_upgrade(
        &self,
        upgrade: bool,
    ) -> Result<(), StorageError> {
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

        let prompt = markdown_for_key(self.prompts.agent_init_key).ok_or_else(|| {
            StorageError::Parse(format!(
                "unknown prompt key: {}",
                self.prompts.agent_init_key
            ))
        })?;

        if upgrade
            && let Some(updated) = replace_lazytask_important_block(&content, self.prompts, prompt)
        {
            fs::write(target_path, updated)?;
            return Ok(());
        }

        if has_lazytask_important_block(&content, self.prompts) {
            return Ok(());
        }

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
            if block_mentions_lazytask(block) {
                return true;
            }
            search_from = end + prompts.important_block_end.len();
        } else {
            break;
        }
    }
    false
}

fn replace_lazytask_important_block(
    content: &str,
    prompts: crate::config::PromptConfig,
    prompt_markdown: &str,
) -> Option<String> {
    let mut search_from = 0usize;
    while let Some(start_rel) = content[search_from..].find(prompts.important_block_start) {
        let marker_start = search_from + start_rel;
        let body_start = marker_start + prompts.important_block_start.len();
        if let Some(end_rel) = content[body_start..].find(prompts.important_block_end) {
            let body_end = body_start + end_rel;
            let marker_end = body_end + prompts.important_block_end.len();
            let block = &content[body_start..body_end];
            if block_mentions_lazytask(block) {
                let updated = format!(
                    "{}{}\n{}\n{}{}",
                    &content[..marker_start],
                    prompts.important_block_start,
                    prompt_markdown.trim_matches('\n'),
                    prompts.important_block_end,
                    &content[marker_end..]
                );
                if updated != content {
                    return Some(updated);
                }
                return None;
            }
            search_from = marker_end;
        } else {
            break;
        }
    }
    None
}

fn block_mentions_lazytask(block: &str) -> bool {
    let lowered = block.to_lowercase();
    lowered.contains("lazytask") || block.contains("`lt`")
}
