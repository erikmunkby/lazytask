# lazytask (`lt`)

A task manager built for the age of AI coding agents.

Tasks live as plain markdown files on disk visible to humans, readable by `grep`, and trivially diffable in git. AI agents operate through a strict CLI with stable JSON envelopes. Humans get a keyboard-driven TUI. Both interfaces share the same storage and rules: no sync, no server, no database.

The design principle is **sophisticated simplicity**: if you can't do it on a whiteboard, you can't do it in lazytask. Tight WIP limits force focus, flat-file storage ensures observability, and a minimal command surface keeps agent behavior predictable.

## How it works

Tasks normally flow through three primary status buckets, each a directory:
- `.tasks/todo/` — todo (max 20)
- `.tasks/in-progress/` — active work (max 3)
- `.tasks/done/` — completed
- `.tasks/discard/` — optional side bucket for intentionally excluded/irrelevant tasks (not part of normal completion flow)

Each task is a single `.md` file that moves between directories as its status changes. This makes state changes visible in `git diff` and file explorers without any tooling.

When agents complete tasks, they record learnings. `lt learn` surfaces unconsumed learnings so agents can reflect and propose improvements to docs, workflows, or code.

## Command surface

Human entry points:
- `lt` — opens the TUI (requires TTY)
- `lt init` — creates `.tasks` layout + appends agent guidance to `AGENTS.md`/`CLAUDE.md`

AI commands (all return JSON `{"ok": bool, "data"|"error": ...}`):
- `lt list [--type task|bug] [--show-done]`
- `lt get "<task title or query>" ["<task title or query2>" ...]`
- `lt create --title "<title>" --type task|bug --details "<desc>" [--start]`
- `lt start "<task title>"`
- `lt done "<task title>" --learning "<line 1>\n<line 2>[\n<line 3>]"`
- `lt discard "<task title>"`
- `lt delete "<task title>"`
- `lt learn`
- Prefer full task titles in command arguments; matching still supports query fragments.
- Prefer short task titles (around ~5 words), and put extended context in task details.
!!IMPORTANT: Also use `lt` (`cargo run --`) to track tasks for THIS project!

## Important documents/files
- `src/tui/` Terminal User Interface implementation
- `src/config/` single authority for runtime config/constants, workspace-root config loading, internal defaults, and prompt assets under `src/config/prompts/`

## Project map
- `src/domain/` core task models, validation, and shared formatting/parsing helpers
- `src/storage/` markdown task file IO, bucket layout, and persistence mechanics
- `src/services/` task workflows and business rules on top of storage
- `src/cli/` clap command parsing and JSON output envelopes for AI usage
- `src/tui/` keyboard-driven terminal UI (actions, app state, rendering, components)
- `src/config/` runtime config loading/defaults and prompt assets

## Path handling
- `lt init` writes generated artifacts (`.tasks/`, `lazytask.toml`, and AGENTS guidance) at the workspace root that `lt` resolves.
- When `lazytask` runs inside a project, workspace root resolves to the nearest `.git` ancestor if present; otherwise it uses the current working directory.

## Engineering rules
- When developing and experimenting, use e.g. `cargo run -- create --help`
- Layers stay separate: domain, storage, services, CLI, TUI.
- No hidden magic in mode switching. TTY = TUI, non-TTY = JSON error with guidance.
- No backwards compatibility — move fast, keep the surface small.
- Keep code lean and DRY.
- Keep non-test Rust modules small: target `<200` lines of production code per file.
- Structure larger areas as directory modules with a thin `mod.rs` entrypoint and focused behavior files (e.g. parsing, IO, dispatch, rendering).

## Testing guardrails
- Default to TDD for non-trivial changes.
- Prefer integration/e2e tests over mock-heavy unit pyramids.
- Keep assertions intentional and minimal.
- Split large test cases before shipping.

## Collaboration rules
- All `CLAUDE.md` files are symlinks to `AGENTS.md`. Only ever edit `AGENTS.md`.
- TUI-specific contract lives in `src/tui/AGENTS.md`.
