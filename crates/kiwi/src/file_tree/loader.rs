use std::fs;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

use super::ignore::is_default_ignored;
use super::node::DirectoryEntry;

#[cfg(test)]
const MAX_EXPAND_DEPTH: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryLoadResult {
    pub children: Vec<DirectoryEntry>,
    pub error: Option<String>,
}

pub fn read_directory_children(path: &Path) -> DirectoryLoadResult {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => {
            return DirectoryLoadResult {
                children: Vec::new(),
                error: Some(err.to_string()),
            };
        }
    };

    let mut children = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };

        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        let name = entry.file_name().to_string_lossy().into_owned();
        if is_default_ignored(&name) {
            continue;
        }
        children.push(DirectoryEntry {
            path: entry.path(),
            name,
            is_dir: file_type.is_dir(),
        });
    }

    sort_directory_entries(&mut children);
    DirectoryLoadResult {
        children,
        error: None,
    }
}

pub fn sort_directory_entries(children: &mut [DirectoryEntry]) {
    children.sort_by(|left, right| match (left.is_dir, right.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => left
            .name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase()),
    });
}

#[cfg(test)]
fn detect_symlink_loop(root: &Path, target: &Path) -> Option<String> {
    let mut seen = Vec::<PathBuf>::new();
    let mut current = Some(target);
    let mut depth = 0usize;

    while let Some(path) = current {
        if depth > MAX_EXPAND_DEPTH {
            return Some(format!(
                "directory depth exceeds {MAX_EXPAND_DEPTH} under {}",
                root.display()
            ));
        }

        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if seen.iter().any(|visited| visited == &canonical) {
            return Some(format!("symlink loop detected at {}", path.display()));
        }
        seen.push(canonical);

        if path == root {
            break;
        }

        current = path.parent();
        depth += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;

    #[test]
    fn read_directory_children_sorts_dirs_before_files_case_insensitive() {
        let temp = std::env::temp_dir().join(format!("kiwi-file-tree-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("Zebra")).expect("mkdir");
        fs::create_dir_all(temp.join("alpha")).expect("mkdir");
        fs::write(temp.join("Beta.txt"), "beta").expect("write");
        fs::write(temp.join("a.txt"), "a").expect("write");

        let result = read_directory_children(&temp);
        assert!(result.error.is_none());
        let names: Vec<_> = result
            .children
            .iter()
            .map(|entry| entry.name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "Zebra", "a.txt", "Beta.txt"]);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn read_directory_children_skips_default_ignored_names() {
        let temp =
            std::env::temp_dir().join(format!("kiwi-file-tree-ignore-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join(".git")).expect("mkdir");
        fs::create_dir_all(temp.join("node_modules")).expect("mkdir");
        fs::create_dir_all(temp.join("target")).expect("mkdir");
        fs::create_dir_all(temp.join("src")).expect("mkdir");
        fs::write(temp.join("Cargo.toml"), "pkg").expect("write");

        let result = read_directory_children(&temp);
        assert!(result.error.is_none());
        let names: Vec<_> = result
            .children
            .iter()
            .map(|entry| entry.name.as_str())
            .collect();
        assert_eq!(names, vec!["src", "Cargo.toml"]);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn read_directory_children_reports_permission_errors() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let temp =
                std::env::temp_dir().join(format!("kiwi-file-tree-denied-{}", std::process::id()));
            let _ = fs::remove_dir_all(&temp);
            fs::create_dir(&temp).expect("mkdir");
            let mut permissions = fs::metadata(&temp).expect("metadata").permissions();
            permissions.set_mode(0o000);
            fs::set_permissions(&temp, permissions).expect("chmod");

            let result = read_directory_children(&temp);
            assert!(result.children.is_empty());
            assert!(result.error.is_some());

            let mut permissions = fs::metadata(&temp).expect("metadata").permissions();
            permissions.set_mode(0o700);
            let _ = fs::set_permissions(&temp, permissions);
            let _ = fs::remove_dir_all(temp);
        }
    }

    #[test]
    fn detect_symlink_loop_flags_deep_paths() {
        let root = Path::new("/tmp/kiwi-root");
        let deep = (0..=MAX_EXPAND_DEPTH).fold(PathBuf::from("/tmp/kiwi-root"), |path, index| {
            path.join(format!("dir-{index}"))
        });
        assert!(detect_symlink_loop(root, &deep).is_some());
    }
}
