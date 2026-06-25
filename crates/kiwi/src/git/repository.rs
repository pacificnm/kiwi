use std::path::Path;

use git2::{BranchType, ErrorCode, Repository};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBranchInfo {
    pub branch: String,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitError {
    Open(String),
    Head(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open(message) => write!(f, "failed to open git repository: {message}"),
            Self::Head(message) => write!(f, "failed to read git HEAD: {message}"),
        }
    }
}

impl std::error::Error for GitError {}

pub fn load_branch_info(repo_root: &Path) -> Result<GitBranchInfo, GitError> {
    let repo = Repository::open(repo_root).map_err(|err| GitError::Open(err.to_string()))?;
    read_branch_info(&repo)
}

fn read_branch_info(repo: &Repository) -> Result<GitBranchInfo, GitError> {
    let head = match repo.head() {
        Ok(head) => head,
        Err(err) if err.code() == ErrorCode::UnbornBranch => {
            return Ok(GitBranchInfo {
                branch: unborn_branch_name(repo),
                ahead: 0,
                behind: 0,
            });
        }
        Err(err) => return Err(GitError::Head(err.to_string())),
    };

    let branch = branch_display_name(&head)?;
    let (ahead, behind) = ahead_behind(repo, &head)?;

    Ok(GitBranchInfo {
        branch,
        ahead,
        behind,
    })
}

fn unborn_branch_name(repo: &Repository) -> String {
    repo.find_reference("HEAD")
        .ok()
        .and_then(|head| head.symbolic_target().map(str::to_string))
        .and_then(|target| target.strip_prefix("refs/heads/").map(str::to_string))
        .unwrap_or_else(|| "main".to_string())
}

fn branch_display_name(head: &git2::Reference) -> Result<String, GitError> {
    if head.is_branch() {
        return Ok(head.shorthand().unwrap_or("HEAD").to_string());
    }

    let commit = head
        .peel_to_commit()
        .map_err(|err| GitError::Head(err.to_string()))?;
    let oid = commit.id().to_string();
    let short = oid.get(..7).unwrap_or(&oid);
    Ok(format!("(detached {short})"))
}

fn ahead_behind(repo: &Repository, head: &git2::Reference) -> Result<(u32, u32), GitError> {
    if !head.is_branch() {
        return Ok((0, 0));
    }

    let Some(branch_name) = head.shorthand() else {
        return Ok((0, 0));
    };

    let branch = match repo.find_branch(branch_name, BranchType::Local) {
        Ok(branch) => branch,
        Err(err) if err.code() == ErrorCode::NotFound => return Ok((0, 0)),
        Err(err) => return Err(GitError::Head(err.to_string())),
    };

    let upstream = match branch.upstream() {
        Ok(upstream) => upstream,
        Err(err) if err.code() == ErrorCode::NotFound => return Ok((0, 0)),
        Err(err) => return Err(GitError::Head(err.to_string())),
    };

    let Some(local_oid) = head.target() else {
        return Ok((0, 0));
    };
    let Some(upstream_oid) = upstream.get().target() else {
        return Ok((0, 0));
    };

    let (ahead, behind) = repo
        .graph_ahead_behind(local_oid, upstream_oid)
        .map_err(|err| GitError::Head(err.to_string()))?;

    Ok((usize_to_u32(ahead), usize_to_u32(behind)))
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::*;

    struct TempGitRepo {
        path: PathBuf,
    }

    impl TempGitRepo {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("kiwi-git-test-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create temp dir");
            init_git_repo(&path);
            Self { path }
        }
    }

    impl Drop for TempGitRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn init_git_repo(path: &Path) {
        run_git(path, &["init"]);
        run_git(path, &["config", "user.email", "kiwi@test.local"]);
        run_git(path, &["config", "user.name", "Kiwi Test"]);
    }

    fn run_git(cwd: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .status()
            .expect("run git");
        assert!(
            status.success(),
            "git {:?} failed in {}",
            args,
            cwd.display()
        );
    }

    #[test]
    fn load_branch_info_reads_default_branch() {
        let repo = TempGitRepo::new("default-branch");
        let info = load_branch_info(&repo.path).expect("branch info");
        assert!(
            info.branch == "main" || info.branch == "master",
            "unexpected branch: {}",
            info.branch
        );
        assert_eq!(info.ahead, 0);
        assert_eq!(info.behind, 0);
    }

    #[test]
    fn load_branch_info_reports_ahead_of_upstream() {
        let repo = TempGitRepo::new("ahead");
        fs::write(repo.path.join("README.md"), "hello\n").expect("write");
        run_git(&repo.path, &["add", "README.md"]);
        run_git(&repo.path, &["commit", "-m", "initial"]);
        run_git(&repo.path, &["branch", "-M", "main"]);

        let bare_path = std::env::temp_dir().join(format!(
            "kiwi-git-bare-remote-{}-{}",
            "ahead",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&bare_path);
        let status = Command::new("git")
            .args(["init", "--bare", bare_path.to_str().expect("utf8")])
            .status()
            .expect("bare init");
        assert!(status.success());

        run_git(
            &repo.path,
            &["remote", "add", "origin", bare_path.to_str().expect("utf8")],
        );
        run_git(&repo.path, &["push", "-u", "origin", "main"]);

        fs::write(repo.path.join("second.txt"), "more\n").expect("write");
        run_git(&repo.path, &["add", "second.txt"]);
        run_git(&repo.path, &["commit", "-m", "second"]);

        let info = load_branch_info(&repo.path).expect("branch info");
        assert_eq!(info.branch, "main");
        assert_eq!(info.ahead, 1);
        assert_eq!(info.behind, 0);

        let _ = fs::remove_dir_all(&bare_path);
    }

    #[test]
    fn load_branch_info_fails_for_missing_git_dir() {
        let path = std::env::temp_dir().join(format!("kiwi-git-norepo-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("mkdir");

        let err = load_branch_info(&path).expect_err("missing repo should fail");
        assert!(matches!(err, GitError::Open(_)));

        let _ = fs::remove_dir_all(path);
    }
}
