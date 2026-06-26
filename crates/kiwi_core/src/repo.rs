use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoRoot {
    pub path: PathBuf,
    pub is_git_repo: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoError {
    NotFound(PathBuf),
    NotADirectory(PathBuf),
}

impl fmt::Display for RepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "path does not exist: {}", path.display()),
            Self::NotADirectory(path) => write!(f, "path is not a directory: {}", path.display()),
        }
    }
}

impl std::error::Error for RepoError {}

pub fn resolve_repo_root(path: &Path) -> Result<RepoRoot, RepoError> {
    let absolute = absolutize(path)?;
    let canonical = std::fs::canonicalize(&absolute).map_err(|_| {
        if absolute.exists() {
            RepoError::NotADirectory(absolute)
        } else {
            RepoError::NotFound(absolute)
        }
    })?;

    if !canonical.is_dir() {
        return Err(RepoError::NotADirectory(canonical));
    }

    if let Some(git_root) = find_git_root(&canonical) {
        return Ok(RepoRoot {
            path: git_root,
            is_git_repo: true,
        });
    }

    Ok(RepoRoot {
        path: canonical,
        is_git_repo: false,
    })
}

pub fn warn_if_not_git_repo(repo: &RepoRoot) {
    if !repo.is_git_repo {
        eprintln!(
            "warning: {} is not a git repository; git features will be disabled",
            repo.path.display()
        );
    }
}

fn absolutize(path: &Path) -> Result<PathBuf, RepoError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    let cwd = std::env::current_dir().map_err(|_| RepoError::NotFound(path.to_path_buf()))?;
    Ok(cwd.join(path))
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let git_path = current.join(".git");
        if git_path.is_dir() || git_path.is_file() {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::{find_git_root, resolve_repo_root, RepoError};

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("kiwi-repo-test-{name}"));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn rejects_missing_path() {
        let missing = std::env::temp_dir().join("kiwi-repo-test-missing-dir");
        let err = resolve_repo_root(&missing).expect_err("missing path should fail");
        assert!(matches!(err, RepoError::NotFound(_)));
    }

    #[test]
    fn rejects_file_path() {
        let dir = TempDir::new("file-path");
        let file = dir.path.join("file.txt");
        fs::write(&file, "hello").expect("write file");

        let err = resolve_repo_root(&file).expect_err("file path should fail");
        assert!(matches!(err, RepoError::NotADirectory(_)));
    }

    #[test]
    fn accepts_non_git_directory() {
        let dir = TempDir::new("non-git");
        let repo = resolve_repo_root(&dir.path).expect("directory should resolve");

        assert!(!repo.is_git_repo);
        assert_eq!(
            repo.path,
            fs::canonicalize(&dir.path).expect("canonicalize")
        );
    }

    #[test]
    fn detects_git_root_from_subdirectory() {
        let dir = TempDir::new("git-subdir");
        init_git_repo(&dir.path);
        let nested = dir.path.join("src").join("lib");
        fs::create_dir_all(&nested).expect("create nested dir");

        let repo = resolve_repo_root(&nested).expect("nested path should resolve");
        assert!(repo.is_git_repo);
        assert_eq!(
            repo.path,
            fs::canonicalize(&dir.path).expect("canonicalize")
        );
    }

    #[test]
    fn find_git_root_walks_upward() {
        let dir = TempDir::new("git-walk");
        init_git_repo(&dir.path);
        let nested = dir.path.join("a").join("b");
        fs::create_dir_all(&nested).expect("create nested dir");

        let root = find_git_root(&nested).expect("git root should be found");
        assert_eq!(root, fs::canonicalize(&dir.path).expect("canonicalize"));
    }

    fn init_git_repo(path: &Path) {
        let status = Command::new("git")
            .args(["init", path.to_str().expect("utf8 path")])
            .status()
            .expect("run git init");
        assert!(status.success(), "git init should succeed");
    }
}
