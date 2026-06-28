use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitRepo {
    pub root: PathBuf,
}

impl GitRepo {
    /// Discover the repository root by walking up from `path`.
    pub fn discover(path: &Path) -> Result<Self> {
        let repo = git2::Repository::discover(path)
            .context("not inside a git repository")?;
        let root = repo
            .workdir()
            .ok_or_else(|| anyhow::anyhow!("bare repositories are not supported"))?
            .to_path_buf();
        Ok(Self { root })
    }

    pub fn status(&self) -> Result<String> {
        let branch = self.current_branch()?;
        let out = self.run_git(&["status", "--short"])?;
        if out.trim().is_empty() {
            Ok(format!("On branch {branch}\nnothing to commit, working tree clean"))
        } else {
            Ok(format!("On branch {branch}\n{out}"))
        }
    }

    pub fn diff(&self, staged: bool) -> Result<String> {
        let args: &[&str] = if staged {
            &["diff", "--staged"]
        } else {
            &["diff"]
        };
        let out = self.run_git(args)?;
        if out.trim().is_empty() {
            Ok(if staged { "No staged changes.".into() } else { "No unstaged changes.".into() })
        } else {
            Ok(out)
        }
    }

    /// Stage specific files (or all with `--all`).
    pub fn add(&self, files: &[&str], all: bool) -> Result<String> {
        if all {
            self.run_git(&["add", "-A"])?;
            Ok("staged all changes".to_string())
        } else if files.is_empty() {
            bail!("no files specified; pass file paths or set all=true");
        } else {
            let mut args = vec!["add", "--"];
            args.extend_from_slice(files);
            self.run_git(&args)?;
            Ok(format!("staged: {}", files.join(", ")))
        }
    }

    /// Stage `files` (if any) then commit with `message`. Returns the new short SHA.
    pub fn commit(&self, message: &str, files: &[&str]) -> Result<String> {
        if !files.is_empty() {
            let mut args = vec!["add", "--"];
            args.extend_from_slice(files);
            self.run_git(&args)?;
        }
        let out = self.run_git(&["commit", "-m", message])?;
        let sha = self.run_git(&["rev-parse", "--short", "HEAD"])?.trim().to_string();
        let branch = self.current_branch()?;
        // Extract the summary line from `git commit` output (first line)
        let summary = out.lines().next().unwrap_or("committed").trim();
        Ok(format!("{summary}\nsha: {sha}  branch: {branch}"))
    }

    pub fn log(&self, limit: usize) -> Result<String> {
        let n = format!("-{limit}");
        self.run_git(&[
            "log",
            &n,
            "--oneline",
            "--decorate",
        ])
    }

    pub fn current_branch(&self) -> Result<String> {
        let out = self.run_git(&["branch", "--show-current"])?;
        let branch = out.trim().to_string();
        if branch.is_empty() {
            // Detached HEAD
            let sha = self.run_git(&["rev-parse", "--short", "HEAD"])?;
            Ok(format!("HEAD (detached at {})", sha.trim()))
        } else {
            Ok(branch)
        }
    }

    /// Create a new local branch, optionally starting from `from` (branch or SHA).
    pub fn create_branch(&self, name: &str, from: Option<&str>) -> Result<String> {
        if let Some(start) = from {
            self.run_git(&["checkout", "-b", name, start])?;
        } else {
            self.run_git(&["checkout", "-b", name])?;
        }
        Ok(format!("created and switched to branch '{name}'"))
    }

    pub fn checkout(&self, branch: &str) -> Result<String> {
        self.run_git(&["checkout", branch])?;
        Ok(format!("switched to branch '{branch}'"))
    }

    fn run_git(&self, args: &[&str]) -> Result<String> {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .output()
            .context("failed to run git")?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            bail!("git {}: {}", args.first().unwrap_or(&""), stderr.trim());
        }

        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}
