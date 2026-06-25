use std::path::Path;

use git2::{Diff, DiffFormat, ErrorCode, Repository};

use super::types::{DiffLine, DiffLineKind, DiffSource, FileDiffLoadResult};

pub fn load_file_diff(
    repo_root: &Path,
    relative_path: &str,
    source: DiffSource,
    context_lines: u32,
) -> FileDiffLoadResult {
    let repo = match Repository::open(repo_root) {
        Ok(repo) => repo,
        Err(err) => return FileDiffLoadResult::error(relative_path, err.to_string()),
    };

    let mut options = git2::DiffOptions::new();
    options.pathspec(relative_path);
    options.context_lines(context_lines);

    let diff = match source {
        DiffSource::Unstaged => {
            options.include_untracked(true);
            let index = match repo.index() {
                Ok(index) => index,
                Err(err) => return FileDiffLoadResult::error(relative_path, err.to_string()),
            };
            repo.diff_index_to_workdir(Some(&index), Some(&mut options))
        }
        DiffSource::Staged => {
            let tree = match head_tree(&repo) {
                Ok(tree) => tree,
                Err(err) => return FileDiffLoadResult::error(relative_path, err.to_string()),
            };
            let index = match repo.index() {
                Ok(index) => index,
                Err(err) => return FileDiffLoadResult::error(relative_path, err.to_string()),
            };
            repo.diff_tree_to_index(Some(&tree), Some(&index), Some(&mut options))
        }
    };

    match diff {
        Ok(diff) => {
            let mut result = parse_diff(relative_path, &diff);
            if !result.is_binary
                && result.error.is_none()
                && result.lines.is_empty()
                && worktree_file_contains_null(repo_root, relative_path)
            {
                result.is_binary = true;
            }
            result
        }
        Err(err) => FileDiffLoadResult::error(relative_path, err.to_string()),
    }
}

fn worktree_file_contains_null(repo_root: &Path, relative_path: &str) -> bool {
    std::fs::read(repo_root.join(relative_path))
        .ok()
        .is_some_and(|bytes| bytes.contains(&0))
}

fn head_tree(repo: &Repository) -> Result<git2::Tree<'_>, String> {
    match repo.head() {
        Ok(head) => head.peel_to_tree().map_err(|err| err.to_string()),
        Err(err) if err.code() == ErrorCode::UnbornBranch => empty_tree(repo),
        Err(err) => Err(err.to_string()),
    }
}

fn empty_tree(repo: &Repository) -> Result<git2::Tree<'_>, String> {
    let builder = repo.treebuilder(None).map_err(|err| err.to_string())?;
    let oid = builder.write().map_err(|err| err.to_string())?;
    repo.find_tree(oid).map_err(|err| err.to_string())
}

fn parse_diff(path: &str, diff: &Diff) -> FileDiffLoadResult {
    let mut is_binary = diff.deltas().any(|delta| delta.flags().is_binary());

    let mut lines = Vec::new();
    let print_result = diff.print(DiffFormat::Patch, |delta, _hunk, line| {
        if delta.flags().is_binary() || line.origin() == 'B' {
            is_binary = true;
            return true;
        }

        let kind = match line.origin() {
            '+' => DiffLineKind::Addition,
            '-' => DiffLineKind::Deletion,
            ' ' => DiffLineKind::Context,
            'F' | 'H' => DiffLineKind::Header,
            _ => DiffLineKind::Header,
        };

        let content = String::from_utf8_lossy(line.content())
            .trim_end_matches('\n')
            .to_string();

        if kind == DiffLineKind::Header && content.contains("Binary files") {
            is_binary = true;
            return true;
        }

        lines.push(DiffLine {
            kind,
            content,
            old_lineno: line.old_lineno(),
            new_lineno: line.new_lineno(),
        });
        true
    });

    if is_binary {
        return FileDiffLoadResult {
            path: path.to_string(),
            lines: Vec::new(),
            is_binary: true,
            error: None,
        };
    }

    match print_result {
        Ok(()) => FileDiffLoadResult {
            path: path.to_string(),
            lines,
            is_binary: false,
            error: None,
        },
        Err(err) => FileDiffLoadResult::error(path, err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::*;
    use crate::diff::DiffLineKind;

    struct TempGitRepo {
        path: PathBuf,
    }

    impl TempGitRepo {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("kiwi-diff-test-{name}-{}", std::process::id()));
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
    fn load_file_diff_shows_unstaged_modifications() {
        let repo = TempGitRepo::new("modified");
        fs::write(repo.path.join("README.md"), "hello\n").expect("write");
        run_git(&repo.path, &["add", "README.md"]);
        run_git(&repo.path, &["commit", "-m", "initial"]);
        fs::write(repo.path.join("README.md"), "hello\nworld\n").expect("modify");

        let result = load_file_diff(&repo.path, "README.md", DiffSource::Unstaged, 3);
        assert!(result.error.is_none());
        assert!(!result.is_binary);
        assert!(result
            .lines
            .iter()
            .any(|line| { line.kind == DiffLineKind::Addition && line.content.contains("world") }));
    }

    #[test]
    fn load_file_diff_shows_staged_additions() {
        let repo = TempGitRepo::new("staged");
        fs::write(repo.path.join("README.md"), "hello\n").expect("write");
        run_git(&repo.path, &["add", "README.md"]);
        run_git(&repo.path, &["commit", "-m", "initial"]);
        fs::write(repo.path.join("new.txt"), "new file\n").expect("write");
        run_git(&repo.path, &["add", "new.txt"]);

        let result = load_file_diff(&repo.path, "new.txt", DiffSource::Staged, 3);
        assert!(result.error.is_none());
        assert!(result.lines.iter().any(|line| {
            line.kind == DiffLineKind::Addition && line.content.contains("new file")
        }));
    }

    #[test]
    fn load_file_diff_marks_binary_files() {
        let repo = TempGitRepo::new("binary");
        fs::write(repo.path.join("README.md"), "hello\n").expect("write");
        run_git(&repo.path, &["add", "README.md"]);
        run_git(&repo.path, &["commit", "-m", "initial"]);

        let bytes = (0_u8..=255).collect::<Vec<_>>();
        fs::write(repo.path.join("binary.bin"), bytes).expect("write binary");
        run_git(&repo.path, &["add", "binary.bin"]);

        let result = load_file_diff(&repo.path, "binary.bin", DiffSource::Staged, 3);
        assert!(result.is_binary, "expected binary file: {:?}", result);
        assert!(result.lines.is_empty());
    }
}
