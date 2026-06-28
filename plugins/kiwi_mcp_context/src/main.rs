mod db;
mod embed;
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
    let cli = Cli::parse();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql:///kiwi_memory?host=/var/run/postgresql".into()
    });

    let embed = EmbedClient::from_env()?;

    if cli.setup_db {
        let mut db = ContextDb::connect(&db_url)?;
        let existing = db.existing_dim()?;
        let needed = embed.dim();
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
    let mut db = ContextDb::connect(&db_url)?;
    let existing = db.existing_dim()?;
    let needed = embed.dim();
    if let Some(dim) = existing {
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
