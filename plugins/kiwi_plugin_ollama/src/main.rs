mod context;
mod mcp_client;
mod ollama;
mod rag;

use anyhow::Result;
use clap::Parser;
use context::ConversationContext;
use mcp_client::McpProcess;
use ollama::{ChatEvent, ChatMessage, ChatToolCall, ChatToolFunction, OllamaTool, OllamaClient};
use rag::RagIndex;
use serde_json::json;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::mpsc;

#[derive(Parser)]
#[command(name = "kiwi-ollama", about = "Ollama coding agent for Kiwi TUI")]
struct Cli {
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

    let url = args
        .url
        .or_else(|| std::env::var("OLLAMA_URL").ok())
        .unwrap_or_else(|| "http://localhost:11434".to_string());

    let model = args
        .model
        .or_else(|| std::env::var("OLLAMA_MODEL").ok())
        .unwrap_or_else(|| "qwen2.5-coder".to_string());

    let embed_model = args
        .embed_model
        .or_else(|| std::env::var("OLLAMA_EMBED_MODEL").ok())
        .unwrap_or_else(|| "nomic-embed-text".to_string());

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

    let client = OllamaClient::new(url.clone(), model.clone(), embed_model);
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

    if !args.no_mcp {
        match McpProcess::spawn(&args.mcp_memory_bin) {
            Ok(proc) => {
                eprintln!("mcp: project memory ready");
                mem_mcp = Some(proc);
            }
            Err(e) => eprintln!("warning: project memory MCP unavailable: {e}"),
        }
        match McpProcess::spawn(&args.mcp_context_bin) {
            Ok(proc) => {
                eprintln!("mcp: context memory ready");
                ctx_mcp = Some(proc);
            }
            Err(e) => eprintln!("warning: context memory MCP unavailable: {e}"),
        }
        match McpProcess::spawn(&args.mcp_git_bin) {
            Ok(proc) => {
                eprintln!("mcp: git/GitHub server ready");
                git_mcp = Some(proc);
            }
            Err(e) => eprintln!("warning: git MCP unavailable: {e}"),
        }
    }

    // Fetch tool list from git MCP (used by Ollama for tool calling)
    let git_tools: Vec<OllamaTool> = git_mcp
        .as_mut()
        .and_then(|mcp| mcp.list_tools().ok())
        .unwrap_or_default();

    println!("kiwi-ollama ready (model: {model}, url: {url})");
    if !git_tools.is_empty() {
        println!("thinking: {} git/GitHub tools available", git_tools.len());
    } else {
        println!("thinking: initializing...");
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
                        "model: {model} | url: {url} | rag: {} | mcp-memory: {} | mcp-context: {} | mcp-git: {} ({} tools) | kb-collections: {kb_filter}",
                        if rag.is_some() { "ready" } else { "indexing..." },
                        if mem_mcp.is_some() { "connected" } else { "unavailable" },
                        if ctx_mcp.is_some() { "connected" } else { "unavailable" },
                        if git_mcp.is_some() { "connected" } else { "unavailable" },
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
                    println!("          GITHUB_TOKEN, GITHUB_REPO");
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
        let mut context_parts: Vec<String> = Vec::new();

        // 1. Project memory
        if let Some(mcp) = mem_mcp.as_mut() {
            println!("running tool: searching project memory");
            let _ = std::io::stdout().flush();
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
            let _ = std::io::stdout().flush();
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
            let _ = std::io::stdout().flush();
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
            let _ = std::io::stdout().flush();
            let hits = index.retrieve(prompt, 5);
            if !hits.is_empty() {
                context_parts.push(format!("[Codebase context]\n{}", hits.join("\n\n")));
            }
        }

        let combined_context = if context_parts.is_empty() {
            None
        } else {
            Some(context_parts.join("\n\n---\n\n"))
        };

        println!("thinking: reasoning about your request");
        let _ = std::io::stdout().flush();

        ctx.push_user(prompt.to_string());
        let base_messages = ctx.build_messages(combined_context.as_deref());

        // ── Agentic tool-call loop ───────────────────────────────────────────
        let tools_ref: Option<&[OllamaTool]> =
            if git_tools.is_empty() { None } else { Some(&git_tools) };

        let mut turn_messages = base_messages;
        let mut full_response = String::new();
        let mut had_error = false;

        'tool_loop: for _round in 0..8 {
            let stream = match client.chat_stream(&turn_messages, tools_ref) {
                Ok(s) => s,
                Err(e) => {
                    println!("error: {e}");
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
                let _ = std::io::stdout().flush();

                let result = git_mcp
                    .as_mut()
                    .map(|mcp| {
                        mcp.call_tool(tool_name, tool_args.clone())
                            .unwrap_or_else(|e| format!("error: {e}"))
                    })
                    .unwrap_or_else(|| "error: git MCP server not connected".to_string());

                // Print first line of result as status
                let summary = result.lines().next().unwrap_or("done").trim();
                println!("completed: {tool_name} → {summary}");
                let _ = std::io::stdout().flush();

                turn_messages.push(ChatMessage::tool_result(result));
            }
            // Continue to next round so model can respond to tool results
        }

        if had_error {
            ctx.pop_last();
            continue;
        }

        if !full_response.is_empty() {
            println!("\ncompleted: response ready");
            let _ = std::io::stdout().flush();
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
