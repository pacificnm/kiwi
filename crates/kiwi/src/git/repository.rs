use std::path::Path;

use git2::{BranchType, ErrorCode, Repository, Status, StatusOptions};

use super::status::{GitFileEntry, GitFileStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBranchInfo {
    pub branch: String,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRepoSnapshot {
    pub branch: GitBranchInfo,
    pub remote_repo: Option<String>,
    pub file_entries: Vec<GitFileEntry>,
}

pub fn parse_remote_repo_slug(url: &str) -> Option<String> {
    let url = url.trim_end_matches(".git");

    let path = if url.contains("://") {
        url.split("://").nth(1)?.split('@').next_back()?
    } else {
        url.split(':').next_back()?
    };

    let segments: Vec<&str> = path.split('/').filter(|segment| !segment.is_empty()).collect();
    if segments.len() >= 2 {
        let owner = segments[segments.len() - 2];
        let repo = segments[segments.len() - 1];
        return Some(format!("{owner}/{repo}"));
    }

    None
}

fn read_remote_repo_name(repo: &Repository) -> Option<String> {
    let remote = repo.find_remote("origin").ok()?;
    let url = remote.url()?;
    parse_remote_repo_slug(url)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitError {
    Open(String),
    Head(String),
    Status(String),
    Branches(String),
    Checkout(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open(message) => write!(f, "failed to open git repository: {message}"),
            Self::Head(message) => write!(f, "failed to read git HEAD: {message}"),
            Self::Status(message) => write!(f, "failed to read git status: {message}"),
            Self::Branches(message) => write!(f, "failed to list branches: {message}"),
            Self::Checkout(message) => write!(f, "failed to checkout branch: {message}"),
        }
    }
}

impl std::error::Error for GitError {}

#[cfg_attr(not(test), allow(dead_code))]
pub fn load_branch_info(repo_root: &Path) -> Result<GitBranchInfo, GitError> {
    let repo = Repository::open(repo_root).map_err(|err| GitError::Open(err.to_string()))?;
    read_branch_info(&repo)
}

pub fn load_repo_snapshot(
    repo_root: &Path,
    show_untracked: bool,
) -> Result<GitRepoSnapshot, GitError> {
    let repo = Repository::open(repo_root).map_err(|err| GitError::Open(err.to_string()))?;
    let branch = read_branch_info(&repo)?;
    let remote_repo = read_remote_repo_name(&repo);
    let file_entries = read_file_statuses(&repo, show_untracked)?;
    Ok(GitRepoSnapshot {
        branch,
        remote_repo,
        file_entries,
    })
}

fn read_file_statuses(
    repo: &Repository,
    show_untracked: bool,
) -> Result<Vec<GitFileEntry>, GitError> {
    let mut options = StatusOptions::new();
    options
        .include_untracked(show_untracked)
        .recurse_untracked_dirs(show_untracked)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);

    let statuses = repo
        .statuses(Some(&mut options))
        .map_err(|err| GitError::Status(err.to_string()))?;

    let mut entries = Vec::new();
    for entry in statuses.iter() {
        let Some(path) = entry.path() else {
            continue;
        };
        let Some(status) = map_git2_status(entry.status()) else {
            continue;
        };
        entries.push(GitFileEntry {
            path: path.replace('\\', "/"),
            status,
        });
    }

    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}

fn map_git2_status(status: Status) -> Option<GitFileStatus> {
    if status.is_ignored() {
        return None;
    }

    if status.is_wt_new() && !status.is_index_new() {
        return Some(GitFileStatus::Untracked);
    }
    if status.is_index_new() {
        return Some(GitFileStatus::Added);
    }
    if status.is_index_deleted() || status.is_wt_deleted() {
        return Some(GitFileStatus::Deleted);
    }
    if status.is_index_modified()
        || status.is_wt_modified()
        || status.is_index_renamed()
        || status.is_wt_renamed()
    {
        return Some(GitFileStatus::Modified);
    }
    if status.is_wt_new() {
        return Some(GitFileStatus::Untracked);
    }

    None
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
    use crate::git::{GitFileEntry, GitFileStatus};

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
    fn parse_remote_repo_slug_supports_https_and_scp_urls() {
        assert_eq!(
            parse_remote_repo_slug("https://github.com/pacificnm/kiwi.git"),
            Some("pacificnm/kiwi".to_string())
        );
        assert_eq!(
            parse_remote_repo_slug("git@github.com:pacificnm/kiwi.git"),
            Some("pacificnm/kiwi".to_string())
        );
        assert_eq!(
            parse_remote_repo_slug("ssh://git@github.com/pacificnm/kiwi.git"),
            Some("pacificnm/kiwi".to_string())
        );
        assert_eq!(parse_remote_repo_slug("bare-repo"), None);
    }

    #[test]
    fn load_file_statuses_lists_modified_and_untracked_files() {
        let repo = TempGitRepo::new("file-status");
        fs::write(repo.path.join("README.md"), "hello\n").expect("write");
        run_git(&repo.path, &["add", "README.md"]);
        run_git(&repo.path, &["commit", "-m", "initial"]);

        fs::write(repo.path.join("README.md"), "changed\n").expect("modify");
        fs::write(repo.path.join("new.txt"), "new\n").expect("write");

        let git_repo = Repository::open(&repo.path).expect("open repo");
        let entries = read_file_statuses(&git_repo, true).expect("statuses");
        assert_eq!(
            entries,
            vec![
                GitFileEntry {
                    path: "README.md".to_string(),
                    status: GitFileStatus::Modified,
                },
                GitFileEntry {
                    path: "new.txt".to_string(),
                    status: GitFileStatus::Untracked,
                },
            ]
        );

        let hidden = read_file_statuses(&git_repo, false).expect("statuses");
        assert_eq!(
            hidden,
            vec![GitFileEntry {
                path: "README.md".to_string(),
                status: GitFileStatus::Modified,
            }]
        );
    }

    #[test]
    fn load_repo_snapshot_includes_branch_and_file_entries() {
        let repo = TempGitRepo::new("snapshot");
        fs::write(repo.path.join("tracked.txt"), "one\n").expect("write");
        run_git(&repo.path, &["add", "tracked.txt"]);
        run_git(&repo.path, &["commit", "-m", "initial"]);
        fs::write(repo.path.join("tracked.txt"), "two\n").expect("modify");

        let snapshot = load_repo_snapshot(&repo.path, true).expect("snapshot");
        assert!(!snapshot.branch.branch.is_empty());
        assert!(snapshot.file_entries.iter().any(|entry| {
            entry.path == "tracked.txt" && entry.status == GitFileStatus::Modified
        }));
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
