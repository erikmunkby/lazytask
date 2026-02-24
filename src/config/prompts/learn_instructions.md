<!---
This prompt is triggered when `lt learn` returns learning entries for review.
Goal: guide the agent to turn learnings into concrete, approval-first improvement proposals.
-->
# !!IMPORTANT: Follow this workflow to convert learnings to knowledge.

Run these phases in order:

## Phase 1 - Investigate
- If `lt learn` returns entries, start this phase immediately.
- Do not ask the user whether to begin investigating.
- Study each learning entry.
- Compare against relevant code and docs in this repository.
- Confirm whether the learning is still valid and actionable.

## Phase 2 - Evaluate
- Continue to this phase immediately after Phase 1.
- Decide what should change and why.
- Typical targets: code behavior, tests, docs/README, AGENTS/CLAUDE guidance, workflow rules, reusable skills, tooling/automation.
- For each finding, classify: small safe change now, or larger/riskier follow-up task.

## Phase 3 - Propose
- Pause here and present proposals before making edits.
- Present concrete proposals to the user before making edits.
- Keep proposals specific: files/areas, expected impact, and risk level.
- Ask for approval on medium/large/risky items.

## Phase 4 - Execute
- Implement only what the user approved.
- Validate changes with relevant tests/checks.
- For deferred work, create explicit tasks with `lt create` so it is not lost.
- Report what changed and what is still pending.

## Completion rule
- Do not run `lt learn --finished` until approved work is done and the user confirms cleanup.
