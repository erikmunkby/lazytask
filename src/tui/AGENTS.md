# TUI Contract

## Keybindings

- Arrow up/down: navigate tasks
- `c`: create a new task (always starts as `todo`)
- `e`: edit selected task (save overwrites selected task)
- `d`: delete selected task
- `u`: undo the last delete in the current TUI session
- `s`: move selected task to `in-progress`
- `x`: move selected task to `done`
- `q`: quit

## Table columns

`Title | Status | Type | Updated | Created`

Status display: `todo`, `->` (in-progress), `done`, `X` (discard, red).

## Architecture

- `app.rs`: state machine (AppState + dispatch). All mutations go through `Action` variants.
- `actions.rs`: action enum and create-flow step enum.
- `render.rs`: pure rendering from `&AppState`. No side effects.
- `components/`: reusable ratatui widgets.
- `mod.rs`: terminal setup/teardown and event loop.

The TUI never touches `task.path` or filesystem paths directly — it passes `&Task` to service methods which delegate to storage.

Preview metadata rendering:
- Status/type colors should match task table colors.
- `created`/`updated` are displayed in local timezone and human format in preview, using a timezone label (for example `Stockholm` or `CET`), while task files remain UTC.

Startup behavior:
- On open, if `.tasks/LEARNINGS.md` non-empty line count exceeds the configured `hints.learn_threshold` value, log a hint suggesting the user ask an AI agent to run `lt learn`.
