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

Pre-existing problems count. If you saw a code smell, a missing abstraction, or a
pattern that doesn't scale: even if you didn't introduce it, capture it.
"Not my diff" is not a reason to stay silent.

Prioritize code and architecture findings over process friction.
Tooling hiccups and workflow notes are fine, but the hard observations about
the code itself are what drive real improvement.

Capture concrete findings. Each learning split with newline.
  lt learn '<task title>' --learning '<what should be better>\n<future consideration>'

Litmus test: does this point at a specific change someone could make?
