mod db;
mod embed;
mod indexer;
mod kb_config;
mod mcp;

use anyhow::{bail, Context, Result};
use clap::Parser;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use db::MemoryDb;
use embed::EmbedClient;

#[derive(Parser)]
#[command(
    name = "kiwi-mcp-memory",
    about = "Project memory MCP server / indexer / knowledge base"
)]
struct Cli {
    /// Create both project_memory and knowledge_base tables and exit
    #[arg(long)]
    setup_db: bool,

    /// Index project documentation into project_memory and exit
    #[arg(long)]
    index: bool,

    /// Project root to index (used with --index)
    #[arg(long, default_value = ".")]
    root: PathBuf,

    /// Index an external knowledge collection and exit
    #[arg(long)]
    index_kb: bool,

    /// Collection name (required with --index-kb in single-collection mode)
    #[arg(long)]
    collection: Option<String>,

    /// Source directory to index (required with --index-kb in single-collection mode)
    #[arg(long)]
    source: Option<PathBuf>,

    /// Comma-separated file extensions to include, e.g. "md,html,rs" (default: all UTF-8 files)
    #[arg(long)]
    extensions: Option<String>,

    /// TOML config listing multiple collections to index in bulk (used with --index-kb)
    #[arg(long)]
    kb_config: Option<PathBuf>,

    /// List indexed knowledge base collections and exit
    #[arg(long)]
    list_collections: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql:///kiwi_memory?host=/var/run/postgresql".into());

    let embed = EmbedClient::from_env()?;
    let needed = embed.dim();

    // ── --setup-db ────────────────────────────────────────────────────────────
    if cli.setup_db {
        let mut db = MemoryDb::connect(&db_url)?;
        check_dim_compat(&mut db, needed, &embed)?;
        db.setup_schema(needed)?;
        db.setup_knowledge_schema(needed)?;
        eprintln!("schema ready ({needed}-dim, backend: {})", embed.backend_name());
        return Ok(());
    }

    // ── --list-collections ────────────────────────────────────────────────────
    if cli.list_collections {
        let mut db = MemoryDb::connect(&db_url)?;
        let cols = db.list_collections()?;
        if cols.is_empty() {
            eprintln!("no collections indexed yet");
        } else {
            for c in &cols {
                println!("{c}");
            }
        }
        return Ok(());
    }

    // ── --index (project docs) ────────────────────────────────────────────────
    if cli.index {
        let mut db = MemoryDb::connect(&db_url)?;
        ensure_project_schema(&mut db, needed, &embed)?;
        let root = cli.root.canonicalize().context("invalid --root path")?;
        indexer::index_project(&root, &mut db, &embed)?;
        return Ok(());
    }

    // ── --index-kb ────────────────────────────────────────────────────────────
    if cli.index_kb {
        let mut db = MemoryDb::connect(&db_url)?;
        ensure_knowledge_schema(&mut db, needed, &embed)?;

        if let Some(cfg_path) = &cli.kb_config {
            // Bulk mode: read TOML config
            let cfg = kb_config::KbConfig::from_file(cfg_path)?;
            for col in &cfg.collections {
                let exts: Vec<&str> = col
                    .extensions
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .map(String::as_str)
                    .collect();
                let source = PathBuf::from(&col.source);
                let source = source.canonicalize()
                    .with_context(|| format!("invalid source path for collection '{}'", col.name))?;
                indexer::index_knowledge(&col.name, &source, &mut db, &embed, &exts)?;
            }
        } else {
            // Single-collection mode
            let collection = cli.collection.as_deref()
                .ok_or_else(|| anyhow::anyhow!("--collection is required with --index-kb (or use --kb-config)"))?;
            let source = cli.source.as_ref()
                .ok_or_else(|| anyhow::anyhow!("--source is required with --index-kb (or use --kb-config)"))?;
            let source = source.canonicalize().context("invalid --source path")?;

            let ext_str = cli.extensions.as_deref().unwrap_or("");
            let exts: Vec<&str> = if ext_str.is_empty() {
                vec![]
            } else {
                ext_str.split(',').map(str::trim).collect()
            };

            indexer::index_knowledge(collection, &source, &mut db, &embed, &exts)?;
        }
        return Ok(());
    }

    // ── MCP server mode ───────────────────────────────────────────────────────
    let mut db = MemoryDb::connect(&db_url)?;
    check_dim_compat(&mut db, needed, &embed)?;

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.context("stdin read error")?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = mcp::handle_line(&line, &mut db, &embed) {
            writeln!(out, "{response}").context("stdout write error")?;
            out.flush().context("stdout flush error")?;
        }
    }

    Ok(())
}

/// Abort if the project_memory table exists with the wrong dimension.
fn check_dim_compat(db: &mut MemoryDb, needed: usize, embed: &EmbedClient) -> Result<()> {
    if let Some(dim) = db.existing_dim()? {
        if dim != needed {
            bail!(
                "project_memory dimension mismatch: table has {dim}-dim vectors but \
                 {} backend produces {needed}-dim; drop and recreate the table or switch backends",
                embed.backend_name()
            );
        }
    }
    Ok(())
}

/// Create project_memory schema if it doesn't exist; abort on dimension mismatch.
fn ensure_project_schema(db: &mut MemoryDb, needed: usize, embed: &EmbedClient) -> Result<()> {
    check_dim_compat(db, needed, embed)?;
    if db.existing_dim()?.is_none() {
        db.setup_schema(needed)?;
    }
    Ok(())
}

/// Create knowledge_base schema if it doesn't exist; abort on dimension mismatch.
fn ensure_knowledge_schema(db: &mut MemoryDb, needed: usize, embed: &EmbedClient) -> Result<()> {
    if let Some(dim) = db.existing_knowledge_dim()? {
        if dim != needed {
            bail!(
                "knowledge_base dimension mismatch: table has {dim}-dim vectors but \
                 {} backend produces {needed}-dim; drop and recreate the table or switch backends",
                embed.backend_name()
            );
        }
    } else {
        db.setup_knowledge_schema(needed)?;
    }
    Ok(())
}
