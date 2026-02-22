# lazytask `lt`

`lt` is an AI-first local task manager with two explicit interfaces:

- Human interface: keyboard-driven TUI
- AI interface: fixed-parameter CLI commands with stable JSON envelopes

## Why use `lt`

- It is observable: tasks are plain markdown files on disk.
- It is fast: low-friction TUI for human operation.
- It is reliable for automation: strict AI command surface and machine-readable outputs.
- It enforces focus: simple status buckets with WIP limits.

## Install

`lazytask` is not published yet. Use these placeholders for future release channels:

```bash
# Homebrew (placeholder)
brew tap <ORG>/<TAP>
brew install lazytask

# Cargo (placeholder)
cargo install lazytask

# Direct binary (placeholder)
curl -L <RELEASE_URL>/lazytask -o lazytask
chmod +x lazytask
mv lazytask /usr/local/bin/lt
```

Current pre-release install from source:

```bash
git clone <REPO_URL>
cd lazytask
cargo install --path . --force
# Or, if you have taskfile installed
task install
```

## Quick start

```bash
lt init
lt
```

`lt init` creates the `.tasks` layout and appends `lt` usage guidance to `AGENTS.md` (or `CLAUDE.md` if `AGENTS.md` is missing).
It also creates `lazytask.toml` if missing.

In the TUI:

- `Up/Down`: navigate tasks
- `c`: create
- `d`: delete
- `u`: undo last delete
- `s`: move selected task to `in-progress`
- `x`: move selected task to `done`
- `q`: quit

## Commands

Human commands:

- `lt`
- `lt init`

AI commands:

- `lt list [--status todo|in-progress|done] [--type task|bug]` (discarded tasks are omitted)
- `lt get <query>...`
- `lt create --title <title> --type task|bug --details <text> [--start]`
- `lt start <query>`
- `lt done <query> --learning "<line1>\n<line2>[\n<line3>]"`
- `lt discard <query>`
- `lt delete <query>`
- `lt learn`

AI response envelope:

```json
{"ok":true,"data":{}}
{"ok":false,"error":{"code":"<machine_code>","message":"<human_message>","details":{}}}
```

## Data on disk

`lt` stores tasks and learning state in your workspace:

- `.tasks/todo/`
- `.tasks/in-progress/`
- `.tasks/done/`
- `.tasks/discard/` (optional side bucket for irrelevant/duplicate tasks)
- `.tasks/LEARNINGS.md`
- `lazytask.toml` (configuration)

Task files are markdown with metadata headers and a details block.

## Configuration

`lazytask.toml` supports user-editable runtime settings:

```toml
[limits]
todo = 20
in_progress = 3

[hints]
learn_threshold = 35
```