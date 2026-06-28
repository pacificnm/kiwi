mod git;
mod github;
mod mcp;

use anyhow::{Context, Result};
use clap::Parser;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kiwi-mcp-git", about = "Git and GitHub MCP server for Kiwi")]
struct Cli {
    /// Path to the git repository (defaults to current directory)
    #[arg(long, default_value = ".")]
    repo: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo_path = cli.repo.canonicalize().context("invalid --repo path")?;
    let git_repo = git::GitRepo::discover(&repo_path)?;

    // GitHub client is optional — tools degrade gracefully if env vars are absent
    let github = github::GitHubClient::from_env().ok();
    if github.is_none() {
        eprintln!(
            "warning: GITHUB_TOKEN or GITHUB_REPO not set — GitHub tools will be unavailable"
        );
    }

    let mut ctx = mcp::Ctx { git: git_repo, github };

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
