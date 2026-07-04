//! `kiwi chat` — send a prompt via the configured AI provider.

use std::io::{self, Read};

use clap::{Arg, ArgAction, ArgMatches, Command};
use nest_ai::AiService;
use nest_cli::AsyncCliCommand;
use nest_core::AppContext;
use nest_error::NestResult;

use crate::chat;

/// Async CLI command for AI chat completion.
pub struct ChatCommand;

#[async_trait::async_trait]
impl AsyncCliCommand for ChatCommand {
    fn name(&self) -> &'static str {
        "chat"
    }

    fn about(&self) -> &'static str {
        "Send a prompt to the configured AI provider"
    }

    fn configure(&self, cmd: Command) -> Command {
        cmd.arg(
            Arg::new("message")
                .help("User message to send")
                .required(false),
        )
        .arg(
            Arg::new("stdin")
                .long("stdin")
                .action(ArgAction::SetTrue)
                .help("Read the prompt from stdin"),
        )
    }

    async fn run_async(&self, ctx: &AppContext, matches: &ArgMatches) -> NestResult<()> {
        let ai = ctx.service::<AiService>()?;
        let message = read_message(matches)?;
        let response = chat::complete_user_message(&ai, &message, None).await?;
        println!("{response}");
        Ok(())
    }
}

fn read_message(matches: &ArgMatches) -> NestResult<String> {
    if matches.get_flag("stdin") {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| nest_error::NestError::io(error.to_string()))?;
        let message = buffer.trim().to_string();
        if message.is_empty() {
            return Err(nest_error::NestError::validation(
                "stdin prompt must not be empty",
            ));
        }
        return Ok(message);
    }

    let message = matches
        .get_one::<String>("message")
        .map(String::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if message.is_empty() {
        return Err(nest_error::NestError::validation(
            "message argument is required (or use --stdin)",
        ));
    }
    Ok(message)
}
