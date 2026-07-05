//! `kiwi agent` — tool-using agent loop via MCP.

use std::io::{self, Read};

use clap::{Arg, ArgAction, ArgMatches, Command};
use nest_agent::{AgentEvent, AgentLoop, CancelToken};
use nest_ai::{AiService, ChatMessage};
use nest_cli::AsyncCliCommand;
use nest_config::ConfigService;
use nest_core::AppContext;
use nest_error::NestResult;
use nest_mcp::McpHub;
use tokio::sync::mpsc;

use crate::agent::{mcp_config_error, AgentLoopConfig};

/// Async CLI command for MCP-backed agent runs.
pub struct AgentCommand;

#[async_trait::async_trait]
impl AsyncCliCommand for AgentCommand {
    fn name(&self) -> &'static str {
        "agent"
    }

    fn about(&self) -> &'static str {
        "Run a tool-using agent query via MCP"
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
        let config = ctx.service::<ConfigService>()?;
        let agent_cfg = AgentLoopConfig::from_config_service(&config)?;
        let message = read_message(matches)?;

        let mut hub = McpHub::from_config_file(
            &agent_cfg.mcp_config_path,
            Some(&agent_cfg.mcp_servers),
        )
        .await
        .map_err(|error| {
            nest_error::NestError::network(mcp_config_error(
                &agent_cfg.mcp_config_path,
                &error,
            ))
            .with_module("nest-mcp")
        })?;

        let (tx, mut rx) = mpsc::channel(32);
        let loop_ = AgentLoop::new(ai.clone(), agent_cfg.agent_config());
        let model = Some(agent_cfg.model.clone());
        let cancel = CancelToken::new();

        let run_handle = tokio::spawn(async move {
            loop_
                .run(
                    &mut hub,
                    vec![ChatMessage::user(&message)],
                    model,
                    tx,
                    cancel,
                )
                .await
        });

        let mut final_content = None;
        while let Some(event) = rx.recv().await {
            print_event(&event);
            if let AgentEvent::Finished { content, .. } = &event {
                final_content = Some(content.clone());
            }
            if matches!(event, AgentEvent::Failed(_)) {
                run_handle.await.ok();
                return Err(nest_error::NestError::network(
                    "agent run failed — see stderr for details",
                )
                .with_module("nest-agent"));
            }
        }

        run_handle
            .await
            .map_err(|error| nest_error::NestError::network(error.to_string()))??;

        if let Some(content) = final_content {
            println!("{content}");
        }
        Ok(())
    }
}

fn print_event(event: &AgentEvent) {
    match event {
        AgentEvent::StepStarted { step } => {
            eprintln!("[step {step}]");
        }
        AgentEvent::ToolCallStarted { tool, arguments } => {
            eprintln!("🔧 {tool}({arguments})");
        }
        AgentEvent::ToolCallFinished {
            tool,
            result,
            duration,
        } => {
            eprintln!("   ↳ {tool}: {result} ({duration:?})");
        }
        AgentEvent::ToolCallFailed { tool, error } => {
            eprintln!("   ✗ {tool}: {error}");
        }
        AgentEvent::TextDelta(text) => {
            eprint!("{text}");
        }
        AgentEvent::Finished { .. } => {
            eprintln!();
        }
        AgentEvent::Failed(error) => {
            eprintln!("\nerror: {error}");
        }
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
