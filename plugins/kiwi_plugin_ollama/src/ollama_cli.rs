//! Stream responses via the `ollama run` CLI (matches terminal `ollama run --verbose` feel).

use std::io::{Read, Write};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

/// Run `ollama run <model> [--verbose] <prompt>` and copy stdout straight through.
pub fn stream_run(model: &str, prompt: &str, verbose: bool) -> Result<()> {
    let mut cmd = Command::new("ollama");
    cmd.arg("run").arg(model);
    if verbose {
        cmd.arg("--verbose");
    }
    cmd.arg(prompt);
    cmd.stdout(Stdio::piped()).stderr(Stdio::inherit());
    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to spawn `ollama run {model}` — is ollama installed?"))?;

    let mut stdout = child
        .stdout
        .take()
        .context("ollama run did not provide stdout")?;

    let mut out = std::io::stdout();
    let mut buf = [0u8; 8192];
    loop {
        let count = stdout
            .read(&mut buf)
            .context("failed reading ollama run output")?;
        if count == 0 {
            break;
        }
        out.write_all(&buf[..count])?;
        out.flush()?;
    }

    let status = child.wait().context("failed waiting for ollama run")?;
    if !status.success() {
        anyhow::bail!("ollama run exited with {status}");
    }
    Ok(())
}
