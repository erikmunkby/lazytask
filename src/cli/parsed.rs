use super::ai::run_ai_command;
use super::command::{Cli, Commands};
use super::json_output::{map_error_code, print_json_error, print_json_success};
use crate::config::{self, AppConfig};
use crate::services::{ServiceError, TaskService};
use anyhow::Result;
use std::io::IsTerminal;

/// Executes a parsed CLI command, routing to TUI or strict JSON command handling.
pub(super) fn run_parsed(cli: Cli) -> Result<()> {
    match cli.command {
        None => {
            if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
                let runtime = load_runtime().map_err(anyhow::Error::from)?;
                crate::tui::run(runtime.service, runtime.config.hints.learn_threshold)?;
                Ok(())
            } else {
                print_json_error(
                    "non_tty_requires_command",
                    "no subcommand provided in non-interactive mode",
                    serde_json::json!({
                        "guidance": [
                            "Use `lt list`",
                            "Use `lt get <query>`",
                            "Use `lt create --title ... --type task|bug --details ...`",
                            "Use `lt discard <query> --discard-note ...`"
                        ]
                    }),
                );
                std::process::exit(2);
            }
        }
        Some(Commands::Init { upgrade }) => {
            let runtime = load_runtime().map_err(anyhow::Error::from)?;
            runtime
                .service
                .init_with_upgrade(upgrade)
                .map_err(anyhow::Error::from)?;
            println!(
                "initialized {} directories",
                runtime.config.storage_layout.tasks_dir
            );
            Ok(())
        }
        Some(command) => {
            let runtime = match load_runtime() {
                Ok(runtime) => runtime,
                Err(err) => {
                    let (code, details) = map_error_code(&err);
                    print_json_error(&code, &err.to_string(), details);
                    std::process::exit(1);
                }
            };

            match run_ai_command(&runtime.service, &runtime.config, command) {
                Ok(data) => {
                    print_json_success(data);
                    Ok(())
                }
                Err(err) => {
                    let (code, details) = map_error_code(&err);
                    print_json_error(&code, &err.to_string(), details);
                    std::process::exit(1);
                }
            }
        }
    }
}

struct Runtime {
    service: TaskService,
    config: AppConfig,
}

/// Loads config/service runtime and performs startup retention cleanup.
fn load_runtime() -> Result<Runtime, ServiceError> {
    let app_config = config::load_from_current_dir()?;
    let service = TaskService::new(app_config.clone());
    service.cleanup_expired_terminal_tasks()?;
    Ok(Runtime {
        service,
        config: app_config,
    })
}
