mod context;
mod ollama;
mod rag;

use anyhow::Result;
use clap::Parser;
use context::ConversationContext;
use ollama::OllamaClient;
use rag::RagIndex;
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

    /// Disable RAG codebase indexing
    #[arg(long)]
    no_rag: bool,
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

    let client = OllamaClient::new(url.clone(), model.clone(), embed_model);
    let mut ctx = ConversationContext::new();

    println!("kiwi-ollama ready (model: {model}, url: {url})");
    println!("thinking: initializing...");
    println!("Type your prompt and press Enter. Commands: /clear /status /help");
    let _ = std::io::stdout().flush();

    // Start background RAG indexing
    let (rag_tx, rag_rx) = mpsc::channel::<RagIndex>();
    let mut rag: Option<RagIndex> = None;

    if !args.no_rag {
        let rag_client = client.clone();
        let rag_repo = repo.clone();
        let cache_path = cache_dir().join("embeddings.json");
        std::thread::spawn(move || {
            match RagIndex::build_or_load(rag_repo, cache_path, rag_client) {
                Ok(index) => {
                    let _ = rag_tx.send(index);
                }
                Err(e) => {
                    eprintln!("warning: {e}");
                }
            }
        });
    }

    let stdin = std::io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => break, // EOF — PTY closed
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

        // Poll for completed RAG index (non-blocking)
        if rag.is_none() {
            if let Ok(index) = rag_rx.try_recv() {
                eprintln!("RAG index ready — codebase context available");
                rag = Some(index);
            }
        }

        // Handle slash commands
        match prompt {
            "/clear" => {
                ctx.clear();
                println!("completed: context cleared");
                let _ = std::io::stdout().flush();
                continue;
            }
            "/status" => {
                println!(
                    "model: {model} | url: {url} | rag: {}",
                    if rag.is_some() { "ready" } else { "indexing..." }
                );
                let _ = std::io::stdout().flush();
                continue;
            }
            "/help" => {
                println!("Commands: /clear  /status  /help");
                println!("Set OLLAMA_URL, OLLAMA_MODEL, OLLAMA_EMBED_MODEL to configure.");
                let _ = std::io::stdout().flush();
                continue;
            }
            _ => {}
        }

        // Build RAG context for this turn
        let rag_context: Option<String> = rag.as_ref().and_then(|index| {
            println!("running tool: searching codebase");
            let _ = std::io::stdout().flush();
            let hits = index.retrieve(prompt, 5);
            if hits.is_empty() {
                None
            } else {
                Some(hits.join("\n\n"))
            }
        });

        println!("thinking: reasoning about your request");
        let _ = std::io::stdout().flush();

        ctx.push_user(prompt.to_string());

        let messages = ctx.build_messages(rag_context.as_deref());

        match client.chat_stream(&messages) {
            Err(e) => {
                println!("error: {e}");
                let _ = std::io::stdout().flush();
                ctx.pop_last(); // undo push_user so history stays clean
                continue;
            }
            Ok(stream) => {
                let mut full_response = String::new();
                let mut had_error = false;

                for token in stream {
                    match token {
                        Ok(text) => {
                            print!("{text}");
                            let _ = std::io::stdout().flush();
                            full_response.push_str(&text);
                        }
                        Err(e) => {
                            println!("\nerror: {e}");
                            let _ = std::io::stdout().flush();
                            had_error = true;
                            break;
                        }
                    }
                }

                if !had_error {
                    println!("\ncompleted: response ready");
                    let _ = std::io::stdout().flush();
                    ctx.push_assistant(full_response);
                }
            }
        }
    }

    Ok(())
}

fn cache_dir() -> PathBuf {
    std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs_home().join(".cache")
        })
        .join("kiwi-ollama")
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
