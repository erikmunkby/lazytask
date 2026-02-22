# Contributing to `lt`

Thanks for contributing.

## Development setup

Prerequisites:

- Rust stable toolchain
- `cargo`
- `task` (Taskfile runner)

Setup:

```bash
task setup
```

## Common commands

```bash
task fix # Linting and formatting
task test # Run all tests
task install # Install lazytask globally, useful for testing lazytask in other project
```

When experimenting with lazytask commands in-repo locally, use `cargo run --`.
Do not pass `lt` again after `--` (for example `cargo run -- lt`), because that becomes `lt lt`.
Example:
```sh
cargo run -- # Open TUI
cargo run -- create --help # Show help for task creation
```

## Architecture rules

Keep layer boundaries strict:

- `src/domain`: pure types, validation, time helpers
- `src/storage`: filesystem and markdown parsing/serialization only
- `src/services`: workflow/business rules
- `src/cli`: command routing and JSON output contracts
- `src/tui`: TUI state, actions, rendering

Do not move business rules into CLI or TUI code.

## Pull request checklist

- Code is formatted.
- Clippy passes with warnings denied.
- Tests pass.
- README/AGENTS/docs updated if command behavior changed.
- Changes remain aligned with `CLAUDE.md` and `AGENTS.md`.
