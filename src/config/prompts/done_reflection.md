<!---
Always included in the `lt done` response next_step field.
Goal: trigger reflection on what could be improved while context is fresh.
-->
You just completed a task. Pause and reflect while the context is fresh.

What should be better in the code you touched?
- Code: duplication, coupling, abstractions that don't fit, edge cases
- Tests: coverage gaps, fragile assertions, paths you verified manually
- Instructions: AGENTS.md guidance that was wrong, missing, or misleading
- DX: friction, confusing naming, files you had to hunt for

Capture concrete findings:
  lt learn '<task title>' --learning '<what should be better>'

Litmus test: does this point at a specific change someone could make?
