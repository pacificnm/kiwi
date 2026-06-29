mod db;
mod embed;
mod env;
mod mcp;

use anyhow::{Context, Result};
use clap::Parser;
use std::io::{self, BufRead, Write};

use db::ContextDb;
use embed::EmbedClient;

#[derive(Parser)]
#[command(name = "kiwi-mcp-context", about = "Agent context memory MCP server")]
struct Cli {
    /// Create or migrate the agent_context_memory table and exit
    #[arg(long)]
    setup_db: bool,
}

fn main() -> Result<()> {
    env::load_mcp_env();
    let cli = Cli::parse();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql:///kiwi_memory?host=/var/run/postgresql".into()
    });

    let mut db = ContextDb::connect(&db_url)?;
    let mut embed = EmbedClient::from_env()?;
    if let Some(table_dim) = db.existing_dim()? {
        embed = embed.align_to_table_dim(table_dim)?;
    }
    let needed = embed.dim();

    if cli.setup_db {
        let existing = db.existing_dim()?;
        if let Some(dim) = existing {
            if dim != needed {
                anyhow::bail!(
                    "schema dimension mismatch: table has {dim}-dim vectors but \
                     {} backend produces {needed}-dim; drop and recreate the table \
                     or switch backends",
                    embed.backend_name()
                );
            }
        }
        db.setup_schema(needed)?;
        eprintln!("schema ready ({needed}-dim, backend: {})", embed.backend_name());
        return Ok(());
    }

    // MCP server mode
    if let Some(dim) = db.existing_dim()? {
        if dim != needed {
            anyhow::bail!(
                "schema dimension mismatch: table has {dim}-dim vectors but \
                 {} backend produces {needed}-dim",
                embed.backend_name()
            );
        }
    }

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
