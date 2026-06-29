use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::command::command_on_path;
use super::types::{RepoMilestone, RepoMilestonesLoadResult};

pub fn load_repo_milestones(repo_root: &Path, command: &str) -> RepoMilestonesLoadResult {
    if !command_on_path(command) {
        return RepoMilestonesLoadResult {
            milestones: Vec::new(),
            error: Some(format!("GitHub CLI ({command}) not found on PATH")),
        };
    }

    let repo = match repo_name_with_owner(repo_root, command) {
        Ok(repo) => repo,
        Err(message) => {
            return RepoMilestonesLoadResult {
                milestones: Vec::new(),
                error: Some(message),
            };
        }
    };

    let endpoint = format!("repos/{repo}/milestones?state=open&per_page=100");
    let output = Command::new(command)
        .args(["api", &endpoint])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(result) if result.status.success() => match parse_repo_milestones_json(&result.stdout) {
            Ok(milestones) => RepoMilestonesLoadResult {
                milestones,
                error: None,
            },
            Err(message) => RepoMilestonesLoadResult {
                milestones: Vec::new(),
                error: Some(message),
            },
        },
        Ok(result) => RepoMilestonesLoadResult {
            milestones: Vec::new(),
            error: Some(format_milestone_list_failure(&result.stderr, &result.stdout)),
        },
        Err(err) => RepoMilestonesLoadResult {
            milestones: Vec::new(),
            error: Some(format!("Failed to run `{command} api`: {err}")),
        },
    }
}

fn repo_name_with_owner(repo_root: &Path, command: &str) -> Result<String, String> {
    let output = Command::new(command)
        .args(["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"])
        .current_dir(repo_root)
        .output()
        .map_err(|err| format!("Failed to run `{command} repo view`: {err}"))?;

    if !output.status.success() {
        return Err(format_milestone_list_failure(&output.stderr, &output.stdout));
    }

    let repo = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if repo.is_empty() {
        return Err("Could not resolve GitHub repository for current directory".to_string());
    }

    Ok(repo)
}

#[derive(Debug, Deserialize)]
struct GhMilestone {
    number: u32,
    title: String,
    description: Option<String>,
}

fn parse_repo_milestones_json(bytes: &[u8]) -> Result<Vec<RepoMilestone>, String> {
    let raw: Vec<GhMilestone> = serde_json::from_slice(bytes)
        .map_err(|err| format!("Invalid gh milestone JSON: {err}"))?;

    Ok(raw
        .into_iter()
        .map(|milestone| RepoMilestone {
            number: milestone.number,
            title: milestone.title,
            description: milestone.description.unwrap_or_default(),
        })
        .collect())
}

fn format_milestone_list_failure(stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();

    if !stderr.is_empty() {
        return stderr;
    }
    if !stdout.is_empty() {
        return stdout;
    }

    "gh milestone list failed".to_string()
}

#[cfg(test)]
mod tests {
    use super::parse_repo_milestones_json;

    #[test]
    fn parse_repo_milestones_json_maps_fields() {
        let json = r#"[{"number":3,"title":"M1","description":"First milestone"}]"#;
        let milestones = parse_repo_milestones_json(json.as_bytes()).expect("parse");
        assert_eq!(milestones.len(), 1);
        assert_eq!(milestones[0].number, 3);
        assert_eq!(milestones[0].title, "M1");
        assert_eq!(milestones[0].description, "First milestone");
    }
}
