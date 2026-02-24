# TUI Keybindings

## Normal/root mode

- Arrow up/down: navigate tasks
- `c`: create a new task (always starts as `todo`)
- `e`: edit selected task (save overwrites selected task)
- `d`: delete selected task
- `u`: undo the last delete in the current TUI session
- `s`: move selected task to `in-progress`
- `x`: move selected task to `done`
- `o`: open selected task file in editor
- `?`: open keybindings overlay
- `q` or `Esc`: quit

## Create/edit modal

- `Tab` or Arrow up/down: switch field
- `Enter`: next field (`Title`/`Type`) or newline (`Details`)
- `Ctrl+S` or `Ctrl+Enter`: save
- `Esc`: cancel and return to normal mode

## Keybindings overlay

- Triggered by `?` from normal/root mode
- `?`, `Esc`, `Enter`, or `q`: close overlay

## Footer hints

Footer hints intentionally show only high-frequency actions:

- `Nav: ↑/↓`
- `Create: c`
- `Edit: e`
- `Open: o`
- `Quit: q`
- `Delete: d (Undo: u)`
- `Keybindings: ?`
