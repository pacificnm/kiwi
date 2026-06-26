use std::path::Path;

pub(crate) fn command_on_path(command: &str) -> bool {
    let path = Path::new(command);
    if path.components().count() > 1 {
        return path.is_file();
    }

    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };

    std::env::split_paths(&paths).any(|dir| dir.join(command).is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_on_path_rejects_missing_command() {
        assert!(!command_on_path("kiwi-missing-gh-command-for-path-check"));
    }
}
