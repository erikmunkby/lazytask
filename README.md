<div align="center">

# lazytask `lt`

**AI-first task management. Human in command.**

Built in Rust. Fast to run, easy to understand.

[![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs/)
[![Rust](https://img.shields.io/badge/Rust-000?logo=rust&logoColor=fff)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[Why lazytask?](#why-lazytask) · [How it works](#how-it-works) · [Getting started](#getting-started) · [Commands](#commands) · [Configuration](#configuration)

</div>

---

## Why lazytask?

AI coding agents are powerful, but they have blind spots. lazytask makes building easier for AI and for you.

### 🧩 Simple by design

Auto-memory, agent swarms, multi-tool orchestration: powerful for some, overkill for many. lazytask takes the opposite approach. Plain markdown files on disk. Readable by `grep`, diffable in git. No sync, no server, no database. AI as a 10x exoskeleton, not a swarm.

> **Sophisticated simplicity**: if you can't do it on a whiteboard, you can't do it in lazytask.

### 🔁 Learnings are prompted, not lost

Most agents finish a task and move on. Nothing is retained. lazytask prompts reflection after every completion: what surprised, what broke, what to do differently. Learnings accumulate until *you* decide it's time to review them. No opaque auto-memory. You trigger the learning cycle, and your agent distills insights into concrete improvements to docs, workflows, or code.

### 🪤 Bugs don't slip through the cracks

Your agent spots a bug while working on something else. Without a place to put it, that bug vanishes. lazytask gives agents a `create` command, so side-findings become tracked tasks instead of forgotten context.

### 🎛️ AI-first, human in control

Agents get a strict CLI with JSON envelopes. You get a keyboard-driven TUI. Same storage, same rules. Stay on top of every task, or let the agent drive. Your call.

## How it works

lazytask has two interfaces that share the same storage:

| | Human | AI agent |
|---|---|---|
| **Interface** | Keyboard-driven TUI | Strict CLI with JSON envelopes |
| **Launch** | `lt` | `lt list`, `lt create`, ... |
| **Workflow** | Navigate, create, move tasks, paste images | Create, start, complete tasks, capture learnings |

Tasks flow through directories. What you see in your file tree *is* the state:

```sh
.tasks/
├── todo/           # up to 20 tasks (configurable)
├── in-progress/    # up to 3 tasks (focus!) (configurable)
├── done/           # completed work
├── discard/        # intentionally excluded
└── LEARNINGS.md    # learnings captured after task completion
```

Each task is a single `.md` file. Moving a task from `todo` to `in-progress` is literally moving a file.

**The feedback loop:** agents do work → record learnings → you trigger review → agents distill insights → better code.

> `.tasks` can be included in git, but we'd discourage it. lazytask tasks are meant to be post-its next to you on your desk.

<div align="center">

![lazytask TUI demo](docs/assets/lazytask_recording.gif)

</div>

## Getting started

### Install

**Homebrew (macOS & Linux):**

```bash
brew install erikmunkby/tap/lazytask
```

**Cargo (all platforms):**

```bash
cargo install lazytask
```

**Binary download:**

Grab the latest binary from [GitHub Releases](https://github.com/erikmunkby/lazytask/releases/latest).

### Quick start

```bash
lt init    # creates .tasks/ layout + config file + agent guidance
lt init --upgrade  # refreshes generated config + agent guidance defaults
lt         # opens the TUI
```

`lt init` also appends usage instructions to your `AGENTS.md` (or `CLAUDE.md`), so your AI agent knows how to use `lt` immediately.
Use `lt init --upgrade` after installing a new lazytask version to refresh generated defaults without overwriting `.tasks/`.

## Commands

### Human commands

| Command | Description |
|---|---|
| `lt` | Open the TUI |
| `lt init` | Initialize lazytask in your project |
| `lt init --upgrade` | Refresh generated config and agent guidance defaults |

### AI commands

All AI commands return small token-efficient JSON: `{"ok": bool, "data": ...}` or `{"ok": false, "error": {...}}`.

| Command | Description |
|---|---|
| `lt list [--type task\|bug] [--show-done]` | List tasks |
| `lt get <query>...` | Get task details |
| `lt create --title '...' --type task\|bug --details '...' [--start]` | Create a task |
| `lt start <query>` | Move task to in-progress |
| `lt done <query>` | Complete task (returns reflection prompt) |
| `lt discard <query> --discard-note '...'` | Discard a task |
| `lt learn --learning '...'` | Record a learning |
| `lt learn --review` | Distill learnings into improvements |

## Configuration

`lazytask.toml` in your project root:

```toml
[limits]
todo = 20          # max tasks in todo
in_progress = 3    # max tasks in progress

[hints]
learn_threshold = 35

[retention]
done_discard_ttl_days = 3    # auto-delete done/discard tasks older than this many days
cleanup_task_assets = true   # delete referenced images when a task is deleted
```

### Custom task directory

Set `LAZYTASK_DIR` to decouple the `.tasks/` directory and `lazytask.toml` from the git root:

```sh
export LAZYTASK_DIR=/path/to/custom/dir
lt init
```

Relative paths resolve against the current working directory, making it work well with [direnv](https://direnv.net/):

```sh
# .envrc
export LAZYTASK_DIR=.lazytask
```

### Git worktrees

lazytask automatically detects git worktrees and resolves back to the main repository root. Tasks are shared across all worktrees with no configuration needed.

## Acknowledgements

lazytask's TUI is built with [ratatui](https://github.com/ratatui/ratatui), and its TUI UX draws heavy inspiration from [lazygit](https://github.com/jesseduffield/lazygit).

## License

MIT
