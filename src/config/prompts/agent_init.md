<!---
This prompt is triggered when `lt init` appends lazytask guidance into `AGENTS.md` or `CLAUDE.md`.
Goal: provide concise, actionable rules for AI agents using `lt` in this repository.
-->
ALWAYS use lazytask (`lt`) for task and bug tracking in this project.
Check existing tasks before starting work (`lt list`). Track all non-trivial work as tasks.
Do NOT create tasks for very quick few-line fixes

Task titles: ~5 words, imperative, specific (e.g. "Add retry logic to API client").
Task details (3-10 sentences): current behavior, expected behavior, and relevant code (`api/auth.py` style paths).
Don't repeat the title. Include enough context that work can start without a research phase.

Commands (all return JSON):
  lt list [--type task|bug] [--show-done]
  lt get '<title>' ['<title2>' ...]
  lt create --title '<title>' --type task|bug --details '<desc>' [--start]
  lt start '<title>'
  lt done '<title>'
  lt discard '<title>' --discard-note '<short why>'  # won't do
  lt learn --review

After `lt done`, follow the `next_step` field in the response to capture learnings.
When `lt learn --review` prompts a review, follow its instructions to convert learnings into project improvements.
