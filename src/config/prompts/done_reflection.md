<!---
Always included in the `lt done` response next_step field.
Goal: trigger reflection on what could be improved while context is fresh.
-->
You just completed a task. Pause and genuinely think about what could be improved —
not just in the code you touched, but in the project as a whole. Don't just check
boxes. Ask yourself what surprised you, what felt wrong, or what you'd do
differently if you started over.

Think beyond the immediate task too. Did you notice patterns across the codebase
that don't scale? Architectural decisions that will cause pain later? Gaps in
testing strategy, documentation, or developer experience that go beyond your diff?

Pre-existing problems count. "Not my diff" is not a reason to stay silent.
Prioritize code and architecture findings over process friction.

Examples of the kind of thing worth capturing (not a checklist — think beyond these):
- An abstraction that doesn't fit, or one that's missing
- A test gap you worked around or verified manually
- AGENTS.md guidance that was wrong or misleading
- Confusing naming or files you had to hunt for

Capture concrete findings. Each learning split with newline.
  lt learn --learning '<what should be better>\n<future consideration>'

Litmus test: does this point at a specific change someone could make?
