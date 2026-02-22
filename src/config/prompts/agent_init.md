<!---
This prompt is triggered when `lt init` appends lazytask guidance into `AGENTS.md` or `CLAUDE.md`.
Goal: provide concise, actionable rules for AI agents using `lt` in this repository.
-->
ALWAYS use lazytask (`lt`) for task and bug tracking in this project.
Check existing tasks before starting work (`lt list`). Track all non-trivial work as tasks.
Do NOT create tasks for very quick few-line fixes

Task titles: ~5 words, imperative, specific (e.g. "Add retry logic to API client").
Task details: what and why, not how. Include acceptance criteria when useful.
One task = one completable unit of work. Split larger efforts.

Commands (all return JSON):
  lt list [--type task|bug] [--show-done]
  lt get "<title>" ["<title2>" ...]
  lt create --title "<title>" --type task|bug --details "<desc>" [--start]
  lt start "<title>"
  lt done "<title>" --learning "<learning>"
  lt discard "<title>"  # won't do, removes from active lists
  lt learn

Learnings (required with `lt done`):
A learning captures what you'd tell the next agent working here.
- Lead with the concrete takeaway: what happened and what to do differently.
- When there's a deeper insight, add it: what assumption was wrong? What pattern should change?
- Bad: "Fixed the bug."
  Good: "Race condition in cache invalidation — add integration test for concurrent writes."
When `lt learn` prompts a review, follow its instructions to convert learnings into project improvements.
