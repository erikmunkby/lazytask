const AGENT_INIT_PROMPT: &str = include_str!("prompts/agent_init.md");
const LEARN_INSTRUCTIONS_MARKDOWN: &str = include_str!("prompts/learn_instructions.md");
const LEARN_THRESHOLD_HINT_MARKDOWN: &str = include_str!("prompts/learn_threshold_hint.md");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromptConfig {
    pub agent_init_key: &'static str,
    pub agent_init_path: &'static str,
    pub learn_instructions_key: &'static str,
    pub learn_instructions_path: &'static str,
    pub learn_threshold_hint_key: &'static str,
    pub learn_threshold_hint_path: &'static str,
    pub important_block_start: &'static str,
    pub important_block_end: &'static str,
}

pub const DEFAULT_PROMPT_CONFIG: PromptConfig = PromptConfig {
    agent_init_key: "agent_init",
    agent_init_path: "src/config/prompts/agent_init.md",
    learn_instructions_key: "learn_instructions",
    learn_instructions_path: "src/config/prompts/learn_instructions.md",
    learn_threshold_hint_key: "learn_threshold_hint",
    learn_threshold_hint_path: "src/config/prompts/learn_threshold_hint.md",
    important_block_start: "<EXTREMELY_IMPORTANT>",
    important_block_end: "</EXTREMELY_IMPORTANT>",
};

pub fn markdown_for_key(key: &str) -> Option<&'static str> {
    match key {
        "agent_init" => Some(prompt_body(AGENT_INIT_PROMPT)),
        "learn_instructions" => Some(prompt_body(LEARN_INSTRUCTIONS_MARKDOWN)),
        "learn_threshold_hint" => Some(prompt_body(LEARN_THRESHOLD_HINT_MARKDOWN)),
        _ => None,
    }
}

fn prompt_body(markdown: &'static str) -> &'static str {
    if let Some(after_open) = markdown
        .strip_prefix("<!---")
        .or_else(|| markdown.strip_prefix("<!--"))
    {
        if let Some((_, rest)) = after_open.split_once("-->") {
            return rest.trim_start_matches(['\r', '\n']);
        }
    }

    markdown
        .split_once("\n---\n")
        .map(|(_, body)| body)
        .unwrap_or(markdown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_assets_include_metadata_comment() {
        for prompt in [
            AGENT_INIT_PROMPT,
            LEARN_INSTRUCTIONS_MARKDOWN,
            LEARN_THRESHOLD_HINT_MARKDOWN,
        ] {
            assert!(
                prompt.starts_with("<!---"),
                "prompt missing metadata comment header"
            );
            assert!(
                prompt.contains("-->"),
                "prompt missing metadata comment close"
            );
        }
    }

    #[test]
    fn markdown_for_key_returns_only_prompt_body() {
        for key in ["agent_init", "learn_instructions", "learn_threshold_hint"] {
            let prompt = markdown_for_key(key).unwrap();
            assert!(!prompt.starts_with("<!--"));
            assert!(!prompt.contains("<!--"));
            assert!(!prompt.contains("-->"));
        }
    }
}
