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
  Bad: "The login should be faster"
  Good: "Login makes 3 sequential DB calls in api/auth.py handle_login(). Batch into one query. The session table has no index on user_id — check if adding one affects the write path."
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
Capture what you couldn't have known before doing the work.
- Surprises: wrong assumptions, unexpected coupling, things that almost broke.
- Gotchas: non-obvious constraints future agents will hit.
- Architectural insight: connections between components not visible from file structure.
  Bad: "Optimized the login flow and it's faster now."
  Good: "Session table had no index on user_id — adding one sped up reads but slowed the write path in background_jobs/cleanup.py. Added a partial index as compromise."
Litmus test: does this teach something not already in the task title or diff?
When `lt learn` prompts a review, follow its instructions to convert learnings into project improvements.
