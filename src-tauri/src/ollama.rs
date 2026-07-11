//! Ollama HTTP / CLI helpers for the Agent activity sidebar.
//!
//! Lists models via `ollama list` against a remote `OLLAMA_HOST`, and exposes
//! ollama.com account sign-in state via the server's `/api/me` endpoint.

use std::process::Command;

use nest_error::{NestError, NestResult};
use serde::{Deserialize, Serialize};

use crate::agent::augmented_path;

/// One model row from `ollama list`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModel {
    /// Model tag (e.g. `qwen3.5:2b`).
    pub name: String,
    /// Human-readable size when parsed (`2.7 GB`).
    pub size: Option<String>,
    /// Relative modified time when parsed (`4 hours ago`).
    pub modified: Option<String>,
}

/// Ollama.com account status for the configured inference server.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaAuthStatus {
    /// True when `/api/me` reports a signed-in ollama.com user.
    pub signed_in: bool,
    /// Human-readable status (username or reachability hint).
    pub detail: String,
}

#[derive(Debug, Deserialize)]
struct MeResponse {
    name: Option<String>,
}

/// Lists models on the inference server at `host` (`host:port` or full URL).
pub fn list_models(host: &str) -> NestResult<Vec<OllamaModel>> {
    let host = host.trim();
    if host.is_empty() {
        return Err(NestError::validation("ollama host is required"));
    }

    let output = Command::new("ollama")
        .arg("list")
        .env("OLLAMA_HOST", normalize_ollama_host(host))
        .env("PATH", augmented_path())
        .output()
        .map_err(|error| {
            NestError::io(format!(
                "failed to run `ollama list`: {error}. Is Ollama installed locally?"
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(NestError::validation(if stderr.is_empty() {
            format!(
                "`ollama list` failed with status {}",
                output.status.code().unwrap_or(-1)
            )
        } else {
            stderr
        }));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_ollama_list(&stdout))
}

/// Returns whether the Ollama server at `host` is signed in to ollama.com.
pub fn auth_status(host: &str) -> OllamaAuthStatus {
    let base = ollama_api_base(host);
    let url = format!("{base}/api/me");

    match ureq::post(&url).call() {
        Ok(response) if response.status() == 200 => match response.into_json::<MeResponse>() {
            Ok(user) => {
                let name = user.name.unwrap_or_default();
                let detail = if name.is_empty() {
                    "Signed in to ollama.com".into()
                } else {
                    format!("Signed in as {name}")
                };
                OllamaAuthStatus {
                    signed_in: true,
                    detail,
                }
            }
            Err(error) => {
                tracing::warn!(target: "kiwi", %url, %error, "ollama /api/me parse failed");
                OllamaAuthStatus {
                    signed_in: true,
                    detail: "Signed in to ollama.com".into(),
                }
            }
        },
        Ok(response) => OllamaAuthStatus {
            signed_in: false,
            detail: format!(
                "Not signed in to ollama.com (HTTP {})",
                response.status()
            ),
        },
        Err(ureq::Error::Status(401, _)) => OllamaAuthStatus {
            signed_in: false,
            detail: "Not signed in to ollama.com".into(),
        },
        Err(error) => {
            tracing::warn!(target: "kiwi", %url, %error, "ollama auth status check failed");
            OllamaAuthStatus {
                signed_in: false,
                detail: format!("Cannot reach Ollama at {base}: {error}"),
            }
        }
    }
}

/// Signs out of ollama.com on the Ollama server at `host`.
pub fn sign_out(host: &str) -> NestResult<()> {
    run_ollama(host, &["signout"])
}

/// Opens the ollama.com sign-in flow in the system browser via `ollama signin`.
pub fn sign_in(host: &str) -> NestResult<()> {
    let output = ollama_command(host)
        .arg("signin")
        .output()
        .map_err(|error| NestError::io(format!("failed to run `ollama signin`: {error}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    tracing::info!(
        target: "kiwi",
        host = ollama_api_base(host),
        output = combined.trim(),
        "ollama signin finished"
    );

    if combined.contains("already signed in") || combined.contains("You need to be signed in") {
        return Ok(());
    }

    if output.status.success() {
        Ok(())
    } else {
        let message = combined.trim();
        Err(NestError::validation(if message.is_empty() {
            "`ollama signin` failed".to_string()
        } else {
            message.to_string()
        }))
    }
}

fn ollama_command(host: &str) -> Command {
    let mut command = Command::new("ollama");
    command.env("PATH", augmented_path());
    let host = host.trim();
    if !host.is_empty() {
        command.env("OLLAMA_HOST", normalize_ollama_host(host));
    }
    command
}

fn run_ollama(host: &str, args: &[&str]) -> NestResult<()> {
    let output = ollama_command(host)
        .args(args)
        .output()
        .map_err(|error| NestError::io(format!("failed to run ollama: {error}")))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let message = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!(
                "ollama {} failed",
                args.first().copied().unwrap_or("command")
            )
        };
        Err(NestError::validation(message))
    }
}

fn ollama_api_base(host: &str) -> String {
    let host = host.trim();
    if host.is_empty() {
        "http://127.0.0.1:11434".to_string()
    } else {
        normalize_ollama_host(host)
    }
}

/// Ensures the Ollama host has a scheme (`http://192.168.88.10:11434`).
pub fn normalize_ollama_host(host: &str) -> String {
    let trimmed = host.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    }
}

/// Parses the tabular output of `ollama list`.
fn parse_ollama_list(stdout: &str) -> Vec<OllamaModel> {
    stdout
        .lines()
        .skip(1) // header row
        .filter_map(parse_ollama_list_line)
        .collect()
}

fn parse_ollama_list_line(line: &str) -> Option<OllamaModel> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts[0].to_string();
    let (size, modified) = if parts.len() >= 4 && is_size_unit(parts[3]) {
        (
            Some(format!("{} {}", parts[2], parts[3])),
            parts.get(4..).map(|rest| rest.join(" ")),
        )
    } else if parts.len() >= 3 {
        (Some(parts[2].to_string()), parts.get(3..).map(|rest| rest.join(" ")))
    } else {
        (None, None)
    };

    Some(OllamaModel {
        name,
        size,
        modified,
    })
}

fn is_size_unit(unit: &str) -> bool {
    matches!(unit, "B" | "KB" | "MB" | "GB" | "TB")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ollama_list_parses_typical_rows() {
        let stdout = "\
NAME                       ID              SIZE      MODIFIED
qwen3.5:2b                 324d162be6ca    2.7 GB    4 hours ago
nomic-embed-text:latest    0a109f422b47    274 MB    25 hours ago
";
        let models = parse_ollama_list(stdout);
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].name, "qwen3.5:2b");
        assert_eq!(models[0].size.as_deref(), Some("2.7 GB"));
        assert_eq!(models[0].modified.as_deref(), Some("4 hours ago"));
    }

    #[test]
    fn normalize_host_adds_scheme() {
        assert_eq!(
            normalize_ollama_host("192.168.88.10:11434"),
            "http://192.168.88.10:11434"
        );
        assert_eq!(
            normalize_ollama_host("http://server.lan:11434"),
            "http://server.lan:11434"
        );
    }

    #[test]
    fn ollama_api_base_defaults_to_localhost() {
        assert_eq!(ollama_api_base(""), "http://127.0.0.1:11434");
        assert_eq!(
            ollama_api_base("192.168.88.10:11434"),
            "http://192.168.88.10:11434"
        );
    }
}
