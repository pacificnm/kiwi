mod gitea;
mod mcp;

use anyhow::{Context, Result};
use std::io::{self, BufRead, Write};

fn main() -> Result<()> {
    // Gitea client is optional — tools degrade gracefully if env vars are absent
    let gitea = gitea::GiteaClient::from_env().ok();
    if gitea.is_none() {
        eprintln!(
            "warning: GITEA_TOKEN, GITEA_URL, or GITEA_REPO not set — Gitea tools will be unavailable"
        );
    }

    let mut ctx = mcp::Ctx { gitea };

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.context("stdin read error")?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = mcp::handle_line(&line, &mut ctx) {
            writeln!(out, "{response}").context("stdout write error")?;
            out.flush().context("stdout flush error")?;
        }
    }

    Ok(())
}
