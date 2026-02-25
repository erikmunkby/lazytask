mod ai;
mod command;
mod json_output;
mod parsed;

use anyhow::Result;
use clap::{CommandFactory, Parser, error::ErrorKind};
use command::Cli;
use json_output::{print_json_error, wants_ai_json_error};
use serde_json::json;

pub fn run() -> Result<()> {
    match Cli::try_parse() {
        Ok(cli) => parsed::run_parsed(cli),
        Err(err) => {
            if matches!(
                err.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                err.print()?;
                std::process::exit(0);
            }

            if wants_ai_json_error() {
                print_json_error(
                    "invalid_arguments",
                    &err.to_string(),
                    json!({
                        "usage": Cli::command().render_usage().to_string(),
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

            err.print()?;
            std::process::exit(2);
        }
    }
}
