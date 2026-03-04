const AGENT_INIT_PROMPT: &str = include_str!("prompts/agent_init.md");
const DONE_REFLECTION_MARKDOWN: &str = include_str!("prompts/done_reflection.md");
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

/// Returns the effective done-reflection prompt: user override if present, else compiled-in default.
pub fn resolve_done_reflection(user_override: Option<&str>) -> &str {
    match user_override {
        Some(s) if !s.is_empty() => s,
        _ => prompt_body(DONE_REFLECTION_MARKDOWN),
    }
}

/// Renders the `[prompts]` TOML section with comment and triple-quoted default.
pub(crate) fn render_prompts_section() -> String {
    let body = prompt_body(DONE_REFLECTION_MARKDOWN);
    format!(
        "\n[prompts]\n\
         # Shown after `lt done` to trigger reflection while context is fresh.\n\
         # Customize to steer what agents focus on when capturing learnings.\n\
         done_reflection = '''\n\
         {body}\
         '''\n"
    )
}

/// Returns prompt markdown by logical key with metadata headers stripped.
pub fn markdown_for_key(key: &str) -> Option<&'static str> {
    match key {
        "agent_init" => Some(prompt_body(AGENT_INIT_PROMPT)),
        "done_reflection" => Some(prompt_body(DONE_REFLECTION_MARKDOWN)),
        "learn_instructions" => Some(prompt_body(LEARN_INSTRUCTIONS_MARKDOWN)),
        "learn_threshold_hint" => Some(prompt_body(LEARN_THRESHOLD_HINT_MARKDOWN)),
        _ => None,
    }
}

/// Removes frontmatter-style metadata wrappers from embedded prompt assets.
fn prompt_body(markdown: &'static str) -> &'static str {
    if let Some(after_open) = markdown
        .strip_prefix("<!---")
        .or_else(|| markdown.strip_prefix("<!--"))
        && let Some((_, rest)) = after_open.split_once("-->")
    {
        return rest.trim_start_matches(['\r', '\n']);
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
            DONE_REFLECTION_MARKDOWN,
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
        for key in [
            "agent_init",
            "done_reflection",
            "learn_instructions",
            "learn_threshold_hint",
        ] {
            let prompt = markdown_for_key(key).unwrap();
            assert!(!prompt.starts_with("<!--"));
            assert!(!prompt.contains("<!--"));
            assert!(!prompt.contains("-->"));
        }
    }

    #[test]
    fn resolve_done_reflection_returns_default_when_no_override() {
        let result = resolve_done_reflection(None);
        assert!(result.contains("You just completed a task"));
    }

    #[test]
    fn resolve_done_reflection_returns_override_when_present() {
        let result = resolve_done_reflection(Some("Custom reflection prompt"));
        assert_eq!(result, "Custom reflection prompt");
    }

    #[test]
    fn resolve_done_reflection_falls_back_on_empty_override() {
        let result = resolve_done_reflection(Some(""));
        assert!(result.contains("You just completed a task"));
    }

    #[test]
    fn render_prompts_section_contains_expected_structure() {
        let section = render_prompts_section();
        assert!(section.contains("[prompts]"));
        assert!(section.contains("done_reflection = '''"));
        assert!(section.contains("You just completed a task"));
        assert!(section.contains("'''"));
    }
}
