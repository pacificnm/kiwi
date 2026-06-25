use std::path::Path;

use crate::commands::fuzzy_matches;
use crate::file_tree::is_default_ignored;

use super::types::{SearchResult, MAX_SEARCH_RESULTS};

pub fn search_files(repo_root: &Path, query: &str) -> (Vec<SearchResult>, bool) {
    if query.is_empty() {
        return (Vec::new(), false);
    }

    let mut results = Vec::new();
    let mut truncated = false;
    collect_file_matches(repo_root, repo_root, query, &mut results, &mut truncated);
    results.sort_by(|left, right| left.id.cmp(&right.id));
    (results, truncated)
}

fn collect_file_matches(
    repo_root: &Path,
    current: &Path,
    query: &str,
    results: &mut Vec<SearchResult>,
    truncated: &mut bool,
) {
    if *truncated {
        return;
    }

    let entries = match std::fs::read_dir(current) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        if *truncated {
            return;
        }

        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if is_default_ignored(&name) {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            collect_file_matches(repo_root, &path, query, results, truncated);
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let Some(relative) = relative_path(repo_root, &path) else {
            continue;
        };

        if !fuzzy_matches(&relative, query) {
            continue;
        }

        if results.len() >= MAX_SEARCH_RESULTS {
            *truncated = true;
            return;
        }

        results.push(SearchResult::file(path, relative));
    }
}

fn relative_path(repo_root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(repo_root)
        .ok()
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .filter(|relative| !relative.is_empty())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn temp_repo(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("kiwi-search-file-{name}-{}", std::process::id()))
    }

    #[test]
    fn search_files_fuzzy_matches_relative_paths() {
        let root = temp_repo("fuzzy");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).expect("mkdir");
        fs::write(root.join("src/main.rs"), "fn main() {}\n").expect("write");
        fs::write(root.join("README.md"), "# Kiwi\n").expect("write");

        let (results, truncated) = search_files(&root, "mn");
        assert!(!truncated);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "src/main.rs");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn search_files_skips_default_ignored_directories() {
        let root = temp_repo("ignore");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("node_modules/pkg")).expect("mkdir");
        fs::write(root.join("node_modules/pkg/hidden.js"), "x").expect("write");
        fs::write(root.join("visible.txt"), "x").expect("write");

        let (results, _) = search_files(&root, "hid");
        assert!(results.is_empty());

        let _ = fs::remove_dir_all(root);
    }
}
