mod context;
mod env_config;
mod mcp_client;
mod ollama;
mod ollama_cli;
mod rag;

use anyhow::{Context, Result};
use clap::Parser;
use context::ConversationContext;
use mcp_client::McpProcess;
use ollama::{ChatEvent, ChatMessage, ChatToolCall, ChatToolFunction, OllamaTool, OllamaClient};
use rag::RagIndex;
use serde_json::json;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::mpsc;

const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";

#[derive(Parser)]
#[command(name = "kiwi-ollama", about = "Ollama coding agent for Kiwi TUI")]
struct Cli {
    /// Fast streaming via `ollama run` without MCP tools (lower latency).
    #[arg(long)]
    stream: bool,

    /// Deprecated: MCP tools mode is now the default.
    #[arg(long, hide = true)]
    tools: bool,

    /// Pass `--verbose` to `ollama run` (shows token timing stats after each reply).
    #[arg(long, default_value_t = true)]
    verbose: bool,
    /// Ollama base URL (overridden by OLLAMA_URL env var)
    #[arg(long)]
    url: Option<String>,

    /// Chat model (overridden by OLLAMA_MODEL env var)
    #[arg(long)]
    model: Option<String>,

    /// Embedding model for RAG (overridden by OLLAMA_EMBED_MODEL env var)
    #[arg(long)]
    embed_model: Option<String>,

    /// Repository root for RAG file indexing
    #[arg(long, default_value = ".")]
    repo: PathBuf,

    /// Disable local RAG codebase indexing
    #[arg(long)]
    no_rag: bool,

    /// Disable MCP server integration (memory, context, git)
    #[arg(long)]
    no_mcp: bool,

    /// Binary name or path for the project memory MCP server
    #[arg(long, default_value = "kiwi-mcp-memory")]
    mcp_memory_bin: String,

    /// Binary name or path for the context memory MCP server
    #[arg(long, default_value = "kiwi-mcp-context")]
    mcp_context_bin: String,

    /// Binary name or path for the git/GitHub MCP server
    #[arg(long, default_value = "kiwi-mcp-git")]
    mcp_git_bin: String,

    /// Binary name or path for the Gitea MCP server
    #[arg(long, default_value = "kiwi-mcp-gitnexus")]
    mcp_gitnexus_bin: String,

    /// Comma-separated knowledge base collection names to search (default: all)
    #[arg(long, default_value = "")]
    kb_collections: String,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Cli::parse();
    let repo = args
        .repo
        .canonicalize()
        .unwrap_or_else(|_| args.repo.clone());
    env_config::load_repo_env(&repo);

    let url = args
        .url
        .clone()
        .or_else(|| std::env::var("OLLAMA_URL").ok())
        .unwrap_or_else(|| DEFAULT_OLLAMA_URL.to_string());

    let model = args
        .model
        .clone()
        .or_else(|| std::env::var("OLLAMA_MODEL").ok())
        .unwrap_or_else(|| "gpt-oss:20b".to_string());

    let mut client = OllamaClient::new(url.clone(), model.clone(), String::new());
    client
        .resolve_chat_model()
        .with_context(|| format!("failed to resolve chat model for Ollama at {url}"))?;
    let model = client.model.clone();

    if args.stream {
        return run_streaming(&args, model);
    }

    run_with_tools(args, client)
}

fn run_streaming(args: &Cli, model: String) -> Result<()> {
    println!("kiwi-ollama ready (model: {model})");
    println!("Streaming via `ollama run` (no MCP). Type a prompt and press Enter. Commands: /help");
    let _ = std::io::stdout().flush();

    let stdin = std::io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("error: stdin read failed: {e}");
                break;
            }
        }

        let prompt = input.trim();
        if prompt.is_empty() {
            continue;
        }

        if prompt.starts_with('/') {
            if handle_slash_command(prompt)? {
                continue;
            }
        }

        if let Err(e) = ollama_cli::stream_run(&model, prompt, args.verbose) {
            eprintln!("error: {e:#}");
        }
        println!();
        let _ = std::io::stdout().flush();
    }

    Ok(())
}

fn handle_slash_command(prompt: &str) -> Result<bool> {
    let (cmd, _rest) = prompt.split_once(' ').unwrap_or((prompt, ""));
    match cmd {
        "/help" => {
            println!("Commands:");
            println!("  /help   — show this help");
            println!("MCP tools are enabled by default. Use --stream for fast `ollama run` mode.");
            let _ = std::io::stdout().flush();
            Ok(true)
        }
        _ => {
            println!("error: unknown command '{cmd}'. Only /help is available in streaming mode.");
            let _ = std::io::stdout().flush();
            Ok(true)
        }
    }
}

fn run_with_tools(args: Cli, mut client: OllamaClient) -> Result<()> {
    let url = client.base_url.clone();
    let embed_model = args
        .embed_model
        .or_else(|| std::env::var("OLLAMA_EMBED_MODEL").ok())
        .unwrap_or_else(|| "nomic-embed-text".to_string());
    client.embed_model = embed_model.clone();

    // MCP servers read OLLAMA_* from the environment; use the resolved values.
    std::env::set_var("OLLAMA_URL", &url);
    if std::env::var("DATABASE_URL").is_err() {
        std::env::set_var(
            "DATABASE_URL",
            "postgresql:///kiwi_memory?host=/var/run/postgresql",
        );
    }

    let repo = args
        .repo
        .canonicalize()
        .unwrap_or_else(|_| args.repo.clone());

    let kb_collections: Vec<String> = args
        .kb_collections
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if let Err(err) = client.resolve_embed_model() {
        eprintln!("warning: {err:#}");
    }
    let model = client.model.clone();
    let embed_model = client.embed_model.clone();
    std::env::set_var("OLLAMA_MODEL", &model);
    std::env::set_var("OLLAMA_EMBED_MODEL", &embed_model);
    let mut ctx = ConversationContext::new();

    // Session key for context memory
    let session_key = format!(
        "kiwi-ollama:{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    );

    // ── Spawn MCP servers ────────────────────────────────────────────────────
    let mut mem_mcp: Option<McpProcess> = None;
    let mut ctx_mcp: Option<McpProcess> = None;
    let mut git_mcp: Option<McpProcess> = None;
    let mut gitnexus_mcp: Option<McpProcess> = None;

    if !args.no_mcp {
        mem_mcp = spawn_mcp_server("project memory", &args.mcp_memory_bin);
        ctx_mcp = spawn_mcp_server("context memory", &args.mcp_context_bin);
        git_mcp = spawn_mcp_server("git/GitHub", &args.mcp_git_bin);
        gitnexus_mcp = spawn_mcp_server("GitNexus", &args.mcp_gitnexus_bin);
    }

    // Fetch tool lists from git and Gitea MCP servers, merge for Ollama tool calling
    let mut all_tools: Vec<OllamaTool> = git_mcp
        .as_mut()
        .and_then(|mcp| mcp.list_tools().ok())
        .unwrap_or_default();
    if let Some(gt) = gitnexus_mcp.as_mut().and_then(|mcp| mcp.list_tools().ok()) {
        all_tools.extend(gt);
    }
    let git_tools = all_tools;

    println!("kiwi-ollama ready (model: {model}, url: {url}, MCP tools enabled)");
    if !git_tools.is_empty() {
        println!("{} git/GitHub tools available", git_tools.len());
    }
    println!("Type your prompt and press Enter. Commands: /clear /status /save [title] /help");
    let _ = std::io::stdout().flush();

    // ── Background local RAG indexing ────────────────────────────────────────
    let (rag_tx, rag_rx) = mpsc::channel::<RagIndex>();
    let mut rag: Option<RagIndex> = None;

    if !args.no_rag {
        let rag_client = client.clone();
        let rag_repo = repo.clone();
        let cache_path = cache_dir().join("embeddings.json");
        std::thread::spawn(move || {
            match RagIndex::build_or_load(rag_repo, cache_path, rag_client) {
                Ok(index) => { let _ = rag_tx.send(index); }
                Err(e) => eprintln!("warning: {e}"),
            }
        });
    }

    let stdin = std::io::stdin();
    let mut input = String::new();
    let mut last_assistant_response: Option<String> = None;

    loop {
        input.clear();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => { eprintln!("error: stdin read failed: {e}"); break; }
        }

        let prompt = input.trim();
        if prompt.is_empty() { continue; }

        // Poll for completed local RAG index
        if rag.is_none() {
            if let Ok(index) = rag_rx.try_recv() {
                eprintln!("RAG index ready — codebase context available");
                rag = Some(index);
            }
        }

        // ── Slash commands ───────────────────────────────────────────────────
        if prompt.starts_with('/') {
            let (cmd, rest) = prompt.split_once(' ').unwrap_or((prompt, ""));
            match cmd {
                "/clear" => {
                    ctx.clear();
                    last_assistant_response = None;
                    println!("completed: context cleared");
                    let _ = std::io::stdout().flush();
                    continue;
                }
                "/status" => {
                    let kb_filter = if kb_collections.is_empty() {
                        "all".to_string()
                    } else {
                        kb_collections.join(",")
                    };
                    println!(
                        "model: {model} | url: {url} | rag: {} | mcp-memory: {} | mcp-context: {} | mcp-git: {} | mcp-gitnexus: {} | tools: {} | kb-collections: {kb_filter}",
                        if rag.is_some() { "ready" } else { "indexing..." },
                        if mem_mcp.is_some() { "connected" } else { "unavailable" },
                        if ctx_mcp.is_some() { "connected" } else { "unavailable" },
                        if git_mcp.is_some() { "connected" } else { "unavailable" },
                        if gitnexus_mcp.is_some() { "connected" } else { "unavailable" },
                        git_tools.len(),
                    );
                    let _ = std::io::stdout().flush();
                    continue;
                }
                "/save" => {
                    let title = if rest.is_empty() { "Untitled note" } else { rest };
                    match ctx_mcp.as_mut() {
                        None => println!("error: context memory MCP server is not connected"),
                        Some(mcp) => {
                            let content = match last_assistant_response.as_deref() {
                                Some(r) => {
                                    let recent = ctx.last_user_message().unwrap_or("");
                                    format!("User: {recent}\n\nAssistant: {r}")
                                }
                                None => {
                                    println!("error: no response to save yet");
                                    let _ = std::io::stdout().flush();
                                    continue;
                                }
                            };
                            match mcp.call_tool(
                                "save_context_memory",
                                json!({
                                    "content": content,
                                    "title": title,
                                    "session_key": session_key,
                                    "tags": ["kiwi-ollama"]
                                }),
                            ) {
                                Ok(msg) => println!("completed: {msg}"),
                                Err(e) => println!("error: failed to save: {e}"),
                            }
                        }
                    }
                    let _ = std::io::stdout().flush();
                    continue;
                }
                "/help" => {
                    println!("Commands:");
                    println!("  /clear           — clear conversation history");
                    println!("  /status          — show model, RAG, and MCP status");
                    println!("  /save [title]    — save last response to context memory");
                    println!("  /help            — show this help");
                    println!("Env vars: OLLAMA_URL, OLLAMA_MODEL, OLLAMA_EMBED_MODEL,");
                    println!("          DATABASE_URL, EMBED_BACKEND,");
                    println!("          GITHUB_TOKEN, GITHUB_REPO,");
                    println!("          GITEA_TOKEN, GITEA_URL, GITEA_REPO");
                    let _ = std::io::stdout().flush();
                    continue;
                }
                _ => {
                    println!("error: unknown command '{cmd}'. Type /help for commands.");
                    let _ = std::io::stdout().flush();
                    continue;
                }
            }
        }

        // ── Gather context ───────────────────────────────────────────────────
        println!("preparing context for your request");
        let mut context_parts: Vec<String> = Vec::new();

        // 1. Project memory
        if let Some(mcp) = mem_mcp.as_mut() {
            println!("running tool: searching project memory");
            match mcp.call_tool("search_project_memory", json!({ "query": prompt, "limit": 5 })) {
                Ok(text) if !text.is_empty() && text != "No results found." => {
                    context_parts.push(format!("[Project memory]\n{text}"));
                }
                Err(e) => eprintln!("warning: project memory search failed: {e}"),
                _ => {}
            }
        }

        // 2. Knowledge base
        if let Some(mcp) = mem_mcp.as_mut() {
            println!("running tool: searching knowledge base");
            if kb_collections.is_empty() {
                match mcp.call_tool("search_knowledge_base", json!({ "query": prompt, "limit": 5 })) {
                    Ok(text) if !text.is_empty() && text != "No results found." => {
                        context_parts.push(format!("[Knowledge base]\n{text}"));
                    }
                    Err(e) => eprintln!("warning: knowledge base search failed: {e}"),
                    _ => {}
                }
            } else {
                let mut kb_parts: Vec<String> = Vec::new();
                for col in &kb_collections {
                    match mcp.call_tool(
                        "search_knowledge_base",
                        json!({ "query": prompt, "limit": 3, "collection": col }),
                    ) {
                        Ok(text) if !text.is_empty() && text != "No results found." => {
                            kb_parts.push(text);
                        }
                        Err(e) => eprintln!("warning: knowledge base ({col}) failed: {e}"),
                        _ => {}
                    }
                }
                if !kb_parts.is_empty() {
                    context_parts.push(format!("[Knowledge base]\n{}", kb_parts.join("\n\n")));
                }
            }
        }

        // 3. Context memory
        if let Some(mcp) = ctx_mcp.as_mut() {
            println!("running tool: searching context memory");
            match mcp.call_tool(
                "search_context_memory",
                json!({ "query": prompt, "limit": 3, "session_key": "" }),
            ) {
                Ok(text) if !text.is_empty() && text != "No results found." => {
                    context_parts.push(format!("[Context memory]\n{text}"));
                }
                Err(e) => eprintln!("warning: context memory search failed: {e}"),
                _ => {}
            }
        }

        // 4. Local RAG (live source code)
        if let Some(index) = rag.as_ref() {
            println!("running tool: searching codebase");
            let hits = index.retrieve(prompt, 5);
            if !hits.is_empty() {
                context_parts.push(format!("[Codebase context]\n{}", hits.join("\n\n")));
            }
        }

        // 5. Pre-coding analysis — always fetch git status so the model knows the
        //    current branch and which files are modified before reasoning about code.
        if let Some(mcp) = git_mcp.as_mut() {
            println!("running tool: analysing codebase state");
            if let Ok(status) = mcp.call_tool("git_status", json!({})) {
                if !status.is_empty() {
                    context_parts.push(format!("[Code analysis]\n{status}"));
                }
            }
        }

        let combined_context = if context_parts.is_empty() {
            None
        } else {
            Some(context_parts.join("\n\n---\n\n"))
        };

        println!("reasoning about your request");

        ctx.push_user(prompt.to_string());
        let base_messages = ctx.build_messages(combined_context.as_deref());

        // ── Agentic tool-call loop ───────────────────────────────────────────
        let tools_ref: Option<&[OllamaTool]> = if git_tools.is_empty()
            || client.chat_tools_supported == Some(false)
            || !client.model_likely_supports_tools()
        {
            None
        } else {
            Some(&git_tools)
        };

        let mut turn_messages = base_messages;
        let mut full_response = String::new();
        let mut had_error = false;
        let mut called_tools: Vec<String> = Vec::new();

        'tool_loop: for round in 0..8 {
            if round == 0 {
                println!("generating response…");
            } else {
                println!("continuing after tools (step {})…", round + 1);
            }

            let stream = match client.chat_stream(&turn_messages, tools_ref) {
                Ok(s) => s,
                Err(e) => {
                    println!("{e:#}");
                    let _ = std::io::stdout().flush();
                    had_error = true;
                    break;
                }
            };

            let mut turn_text = String::new();
            let mut turn_calls: Vec<(String, serde_json::Value)> = Vec::new();

            for event in stream {
                match event {
                    Ok(ChatEvent::Token(text)) => {
                        print!("{text}");
                        let _ = std::io::stdout().flush();
                        turn_text.push_str(&text);
                    }
                    Ok(ChatEvent::ToolCall { name, arguments }) => {
                        turn_calls.push((name, arguments));
                    }
                    Err(e) => {
                        println!("\nerror: {e}");
                        let _ = std::io::stdout().flush();
                        had_error = true;
                        break 'tool_loop;
                    }
                }
            }

            if turn_calls.is_empty() {
                // Final text response
                full_response = turn_text;
                break;
            }

            // Model wants to call tools — execute them and feed results back
            if !turn_text.is_empty() {
                println!();
            }

            // Add assistant message (may have both text and tool_calls)
            turn_messages.push(ChatMessage {
                role: "assistant".into(),
                content: turn_text.clone(),
                tool_calls: Some(
                    turn_calls
                        .iter()
                        .map(|(name, args)| ChatToolCall {
                            function: ChatToolFunction {
                                name: name.clone(),
                                arguments: args.clone(),
                            },
                        })
                        .collect(),
                ),
            });

            for (tool_name, tool_args) in &turn_calls {
                println!("running tool: {tool_name}");
                called_tools.push(tool_name.clone());

                // Route to the correct MCP server: Gitea tools → gitnexus, rest → git
                let result = if tool_name.starts_with("gitea_") {
                    gitnexus_mcp
                        .as_mut()
                        .map(|mcp| {
                            mcp.call_tool(tool_name, tool_args.clone())
                                .unwrap_or_else(|e| format!("error: {e}"))
                        })
                        .unwrap_or_else(|| "error: Gitea MCP server not connected".to_string())
                } else {
                    git_mcp
                        .as_mut()
                        .map(|mcp| {
                            mcp.call_tool(tool_name, tool_args.clone())
                                .unwrap_or_else(|e| format!("error: {e}"))
                        })
                        .unwrap_or_else(|| "error: git MCP server not connected".to_string())
                };

                // Print first line of result as status
                let summary = result.lines().next().unwrap_or("done").trim();
                println!("completed: {tool_name} → {summary}");

                turn_messages.push(ChatMessage::tool_result(result));
            }
            // Continue to next round so model can respond to tool results
        }

        if had_error {
            ctx.pop_last();
            continue;
        }

        // Post-edit indexing: if any code-modifying tools ran, re-index project memory
        let code_was_modified = called_tools
            .iter()
            .any(|t| matches!(t.as_str(), "git_add" | "git_commit" | "git_checkout"));
        if code_was_modified {
            if let Some(mcp) = mem_mcp.as_mut() {
                println!("running tool: re-indexing project memory after code edits");
                let repo_str = repo.to_string_lossy();
                match mcp.call_tool("index_project", json!({ "root": repo_str.as_ref() })) {
                    Ok(_) => {
                        println!("completed: project memory updated");
                    }
                    Err(e) => eprintln!("warning: post-edit re-index failed: {e}"),
                }
            }
        }

        if !full_response.is_empty() {
            println!("completed: response ready");
        }
        last_assistant_response = Some(full_response.clone());
        ctx.push_assistant(full_response.clone());

        // Auto-save exchange to context memory
        if let Some(mcp) = ctx_mcp.as_mut() {
            let title: String = prompt.chars().take(60).collect();
            let content = format!("User: {prompt}\n\nAssistant: {full_response}");
            if let Err(e) = mcp.call_tool(
                "save_context_memory",
                json!({
                    "content": content,
                    "title": title,
                    "session_key": session_key,
                    "tags": ["kiwi-ollama"]
                }),
            ) {
                eprintln!("warning: context memory auto-save failed: {e}");
            }
        }
    }

    Ok(())
}

fn spawn_mcp_server(label: &str, binary: &str) -> Option<McpProcess> {
    match McpProcess::spawn(binary) {
        Ok(proc) => {
            println!("mcp: {label} ready ({binary})");
            Some(proc)
        }
        Err(err) => {
            println!("warning: {label} MCP unavailable: {err:#}");
            None
        }
    }
}

fn cache_dir() -> PathBuf {
    std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs_home().join(".cache"))
        .join("kiwi-ollama")
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
