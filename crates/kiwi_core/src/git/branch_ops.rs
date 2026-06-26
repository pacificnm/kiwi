use std::path::Path;

use git2::build::CheckoutBuilder;
use git2::{BranchType, Repository};

use super::repository::GitError;

use super::branches::BranchEntry;

pub fn list_local_branches(repo_root: &Path) -> Result<Vec<BranchEntry>, GitError> {
    let repo = Repository::open(repo_root).map_err(|err| GitError::Open(err.to_string()))?;
    let mut entries = Vec::new();

    let branches = repo
        .branches(Some(BranchType::Local))
        .map_err(|err| GitError::Branches(err.to_string()))?;

    for branch in branches {
        let (branch, _) = branch.map_err(|err| GitError::Branches(err.to_string()))?;
        let Some(name) = branch
            .name()
            .map_err(|err| GitError::Branches(err.to_string()))?
        else {
            continue;
        };
        entries.push(BranchEntry {
            name: name.to_string(),
            is_current: branch.is_head(),
        });
    }

    sort_branch_entries(&mut entries);
    Ok(entries)
}

pub fn checkout_local_branch(repo_root: &Path, branch_name: &str) -> Result<(), GitError> {
    let repo = Repository::open(repo_root).map_err(|err| GitError::Open(err.to_string()))?;

    if let Ok(head) = repo.head() {
        if head.is_branch() && head.shorthand() == Some(branch_name) {
            return Ok(());
        }
    }

    let branch = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|err| GitError::Checkout(err.to_string()))?;
    let reference = branch.get();
    let tree = reference
        .peel_to_tree()
        .map_err(|err| GitError::Checkout(err.to_string()))?;

    let mut checkout = CheckoutBuilder::new();
    repo.checkout_tree(&tree.into_object(), Some(&mut checkout))
        .map_err(|err| GitError::Checkout(err.to_string()))?;

    let ref_name = reference
        .name()
        .ok_or_else(|| GitError::Checkout("branch reference has no name".to_string()))?;
    repo.set_head(ref_name)
        .map_err(|err| GitError::Checkout(err.to_string()))?;

    Ok(())
}

fn sort_branch_entries(entries: &mut [BranchEntry]) {
    entries.sort_by(|left, right| match (left.is_current, right.is_current) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => left.name.cmp(&right.name),
    });
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use super::*;

    struct TempGitRepo {
        path: std::path::PathBuf,
    }

    impl TempGitRepo {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir()
                .join(format!("kiwi-branch-test-{name}-{}", std::process::id()));
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
    fn list_local_branches_puts_current_first_then_alphabetical() {
        let repo = TempGitRepo::new("list");
        fs::write(repo.path.join("README.md"), "hello\n").expect("write");
        run_git(&repo.path, &["add", "README.md"]);
        run_git(&repo.path, &["commit", "-m", "initial"]);
        run_git(&repo.path, &["branch", "-M", "main"]);
        run_git(&repo.path, &["branch", "feature-a"]);
        run_git(&repo.path, &["branch", "feature-b"]);
        run_git(&repo.path, &["checkout", "main"]);

        let entries = list_local_branches(&repo.path).expect("branches");
        assert_eq!(entries.len(), 3);
        assert!(entries[0].is_current);
        assert_eq!(entries[0].name, "main");
        assert_eq!(entries[1].name, "feature-a");
        assert_eq!(entries[2].name, "feature-b");
    }

    #[test]
    fn checkout_local_branch_switches_head() {
        let repo = TempGitRepo::new("checkout");
        fs::write(repo.path.join("README.md"), "hello\n").expect("write");
        run_git(&repo.path, &["add", "README.md"]);
        run_git(&repo.path, &["commit", "-m", "initial"]);
        run_git(&repo.path, &["branch", "-M", "main"]);
        run_git(&repo.path, &["branch", "dev"]);

        checkout_local_branch(&repo.path, "dev").expect("checkout");
        let entries = list_local_branches(&repo.path).expect("branches");
        assert!(entries
            .iter()
            .any(|entry| entry.name == "dev" && entry.is_current));
    }

    #[test]
    fn checkout_current_branch_is_no_op() {
        let repo = TempGitRepo::new("noop");
        fs::write(repo.path.join("README.md"), "hello\n").expect("write");
        run_git(&repo.path, &["add", "README.md"]);
        run_git(&repo.path, &["commit", "-m", "initial"]);
        run_git(&repo.path, &["branch", "-M", "main"]);

        checkout_local_branch(&repo.path, "main").expect("checkout");
    }
}
