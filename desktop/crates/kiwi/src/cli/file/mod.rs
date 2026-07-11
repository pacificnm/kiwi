//! `kiwi file` — scoped read, write, update, delete, and list operations.

use std::io::{self, Read};

use clap::{Arg, ArgAction, ArgMatches, Command};
use nest_cli::CliCommand;
use nest_core::AppContext;
use nest_error::NestResult;
use nest_file::{search_files, DirEntry, FileSearchOptions, FileService};
use serde::Serialize;

/// Synchronous CLI command group for workspace file operations.
pub struct FileCommand;

#[derive(Serialize)]
struct ListEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
}

impl CliCommand for FileCommand {
    fn name(&self) -> &'static str {
        "file"
    }

    fn about(&self) -> &'static str {
        "Read, write, update, delete, and list files in the project workspace"
    }

    fn configure(&self, cmd: Command) -> Command {
        cmd.subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(read_command())
            .subcommand(write_command())
            .subcommand(create_command())
            .subcommand(update_command())
            .subcommand(delete_command())
            .subcommand(mkdir_command())
            .subcommand(list_command())
            .subcommand(search_command())
    }

    fn run(&self, ctx: &AppContext, matches: &ArgMatches) -> NestResult<()> {
        let files = ctx.service::<FileService>()?;
        let (name, sub_matches) = matches
            .subcommand()
            .ok_or_else(|| nest_error::NestError::command("missing file subcommand"))?;

        match name {
            "read" => run_read(&files, sub_matches),
            "write" => run_write(&files, sub_matches),
            "create" => run_create(&files, sub_matches),
            "update" => run_update(&files, sub_matches),
            "delete" => run_delete(&files, sub_matches),
            "mkdir" => run_mkdir(&files, sub_matches),
            "list" => run_list(&files, sub_matches),
            "search" => run_search(&files, sub_matches),
            other => Err(nest_error::NestError::command(format!(
                "unknown file subcommand: {other}"
            ))),
        }
    }
}

fn read_command() -> Command {
    Command::new("read")
        .about("Read a UTF-8 text file")
        .arg(
            Arg::new("path")
                .help("Path relative to the project root")
                .required(true),
        )
}

fn write_command() -> Command {
    Command::new("write")
        .about("Write UTF-8 text to a file (creates or overwrites)")
        .arg(
            Arg::new("path")
                .help("Path relative to the project root")
                .required(true),
        )
        .arg(
            Arg::new("content")
                .long("content")
                .help("Text content to write")
                .required(false),
        )
        .arg(
            Arg::new("stdin")
                .long("stdin")
                .action(ArgAction::SetTrue)
                .help("Read content from stdin"),
        )
}

fn create_command() -> Command {
    Command::new("create")
        .about("Create a new empty file")
        .arg(
            Arg::new("path")
                .help("Path relative to the project root")
                .required(true),
        )
}

fn update_command() -> Command {
    Command::new("update")
        .about("Replace text in an existing file")
        .arg(
            Arg::new("path")
                .help("Path relative to the project root")
                .required(true),
        )
        .arg(
            Arg::new("old")
                .long("old")
                .help("Exact text to replace")
                .required(true),
        )
        .arg(
            Arg::new("new")
                .long("new")
                .help("Replacement text")
                .required(true),
        )
        .arg(
            Arg::new("replace_all")
                .long("replace-all")
                .action(ArgAction::SetTrue)
                .help("Replace every occurrence instead of only the first"),
        )
}

fn delete_command() -> Command {
    Command::new("delete")
        .about("Delete a file or directory")
        .arg(
            Arg::new("path")
                .help("Path relative to the project root")
                .required(true),
        )
        .arg(
            Arg::new("recursive")
                .long("recursive")
                .short('r')
                .action(ArgAction::SetTrue)
                .help("Delete a directory and its contents"),
        )
}

fn mkdir_command() -> Command {
    Command::new("mkdir")
        .about("Create a directory and any missing parents")
        .arg(
            Arg::new("path")
                .help("Path relative to the project root")
                .required(true),
        )
}

fn list_command() -> Command {
    Command::new("list")
        .about("List entries in a directory as JSON")
        .arg(
            Arg::new("path")
                .help("Path relative to the project root (default: .)")
                .required(false)
                .default_value("."),
        )
}

fn search_command() -> Command {
    Command::new("search")
        .about("Search for files and directories by path substring")
        .arg(
            Arg::new("query")
                .help("Search terms matched against relative paths")
                .required(true)
                .num_args(1..)
                .trailing_var_arg(true),
        )
        .arg(
            Arg::new("path")
                .long("path")
                .help("Directory to search from (default: .)")
                .default_value("."),
        )
        .arg(
            Arg::new("limit")
                .long("limit")
                .help("Maximum number of matches (default: 50)")
                .default_value("50"),
        )
}

fn run_read(files: &FileService, matches: &ArgMatches) -> NestResult<()> {
    let path = required_path(matches, "path")?;
    let content = files.read_text(path)?;
    print!("{content}");
    Ok(())
}

fn run_write(files: &FileService, matches: &ArgMatches) -> NestResult<()> {
    let path = required_path(matches, "path")?;
    let content = read_content(matches)?;
    nest_agent::write_workspace_file(files, path, &content)?;
    Ok(())
}

fn run_create(files: &FileService, matches: &ArgMatches) -> NestResult<()> {
    let path = required_path(matches, "path")?;
    let summary = nest_agent::create_file(files, path)?;
    eprintln!("{summary}");
    Ok(())
}

fn run_update(files: &FileService, matches: &ArgMatches) -> NestResult<()> {
    let path = required_path(matches, "path")?;
    let old = matches
        .get_one::<String>("old")
        .map(String::as_str)
        .unwrap_or("");
    let new = matches
        .get_one::<String>("new")
        .map(String::as_str)
        .unwrap_or("");
    let replace_all = matches.get_flag("replace_all");

    let mut content = files.read_text(path)?;
    let count = if replace_all {
        let matches = content.matches(old).count();
        if matches == 0 {
            return Err(nest_error::NestError::validation(format!(
                "text to replace was not found in {path}"
            )));
        }
        content = content.replace(old, new);
        matches
    } else {
        if !content.contains(old) {
            return Err(nest_error::NestError::validation(format!(
                "text to replace was not found in {path}"
            )));
        }
        content = content.replacen(old, new, 1);
        1
    };

    files.write_text(path, content)?;
    eprintln!("updated {count} occurrence(s) in {path}");
    Ok(())
}

fn run_delete(files: &FileService, matches: &ArgMatches) -> NestResult<()> {
    let path = required_path(matches, "path")?;
    let recursive = matches.get_flag("recursive");
    let metadata = files.metadata(path)?;

    if metadata.is_dir {
        files.delete_dir(path, recursive)?;
    } else if metadata.is_file {
        if recursive {
            return Err(nest_error::NestError::validation(format!(
                "{path} is a file; omit --recursive to delete files"
            )));
        }
        files.delete_file(path)?;
    } else {
        return Err(nest_error::NestError::validation(format!(
            "{path} is not a regular file or directory"
        )));
    }

    Ok(())
}

fn run_mkdir(files: &FileService, matches: &ArgMatches) -> NestResult<()> {
    let path = required_path(matches, "path")?;
    files.create_dir_all(path)?;
    Ok(())
}

fn run_list(files: &FileService, matches: &ArgMatches) -> NestResult<()> {
    let path = required_path(matches, "path")?;
    let entries = files.list_dir(path)?;
    let payload: Vec<ListEntry> = entries
        .into_iter()
        .map(format_list_entry)
        .collect();
    println!("{}", serde_json::to_string_pretty(&payload).map_err(|error| {
        nest_error::NestError::validation(format!("failed to encode directory listing: {error}"))
    })?);
    Ok(())
}

fn run_search(files: &FileService, matches: &ArgMatches) -> NestResult<()> {
    let query = matches
        .get_many::<String>("query")
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let path = required_path(matches, "path")?;
    let limit = matches
        .get_one::<String>("limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(50);
    let matches = search_files(
        files,
        &FileSearchOptions::for_query(query).with_scope(path, limit),
    )?;
    println!("{}", serde_json::to_string_pretty(&matches).map_err(|error| {
        nest_error::NestError::validation(format!("failed to encode search results: {error}"))
    })?);
    Ok(())
}

fn format_list_entry(entry: DirEntry) -> ListEntry {
    ListEntry {
        name: entry.name,
        path: entry.path.display().to_string(),
        is_dir: entry.metadata.is_dir,
        size: entry.metadata.len,
    }
}

fn required_path<'a>(matches: &'a ArgMatches, name: &str) -> NestResult<&'a str> {
    matches
        .get_one::<String>(name)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| nest_error::NestError::validation(format!("missing {name}")))
}

fn read_content(matches: &ArgMatches) -> NestResult<String> {
    if matches.get_flag("stdin") {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| nest_error::NestError::io(error.to_string()))?;
        return Ok(buffer);
    }

    matches
        .get_one::<String>("content")
        .cloned()
        .ok_or_else(|| {
            nest_error::NestError::validation("either --content or --stdin is required")
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    use nest_file::FileModule;
    use nest_core::AppBuilder;

    fn file_service(root: &std::path::Path) -> FileService {
        let built = AppBuilder::new()
            .module(FileModule::scoped(root))
            .build()
            .unwrap();
        built.context.service::<FileService>().unwrap().clone()
    }

    #[test]
    fn write_read_update_delete_round_trip() {
        let dir = tempdir().unwrap();
        let files = file_service(dir.path());

        files.write_text("notes.txt", "hello world").unwrap();
        assert_eq!(files.read_text("notes.txt").unwrap(), "hello world");

        let mut content = files.read_text("notes.txt").unwrap();
        content = content.replace("world", "kiwi").to_string();
        files.write_text("notes.txt", content).unwrap();
        assert_eq!(files.read_text("notes.txt").unwrap(), "hello kiwi");

        files.delete_file("notes.txt").unwrap();
        assert!(!files.exists("notes.txt").unwrap());
    }

    #[test]
    fn mkdir_list_and_delete_dir() {
        let dir = tempdir().unwrap();
        let files = file_service(dir.path());

        files.create_dir_all("nested/dir").unwrap();
        fs::write(dir.path().join("nested/dir/a.txt"), "a").unwrap();

        let entries = files.list_dir("nested").unwrap();
        assert!(entries.iter().any(|entry| entry.name == "dir"));

        files.delete_dir("nested", true).unwrap();
        assert!(!files.exists("nested").unwrap());
    }
}
