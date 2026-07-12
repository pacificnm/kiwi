//! New Application Wizard - scaffold Nest apps from templates.

use std::fs;
use std::path::Path;
use std::process::Command;

use nest_error::{NestError, NestResult};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime, State};

use crate::workspace::Workspace;

/// Application type for scaffolding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppType {
    /// Desktop GUI app (Tauri + React)
    Gui,
    /// Terminal UI app (Ratatui)
    Tui,
    /// CLI application
    Cli,
    /// System service / background process
    System,
    /// HTTP API server (nest-http-serve)
    ApiServer,
    /// HTTP API server plus a Vite/React web front end
    ApiServerWeb,
}

impl std::fmt::Display for AppType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppType::Gui => write!(f, "gui"),
            AppType::Tui => write!(f, "tui"),
            AppType::Cli => write!(f, "cli"),
            AppType::System => write!(f, "system"),
            AppType::ApiServer => write!(f, "api-server"),
            AppType::ApiServerWeb => write!(f, "api-server-web"),
        }
    }
}

/// Request to scaffold a new application.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldRequest {
    /// Application name (will be used for directory and package name).
    pub name: String,
    /// Type of application to create.
    pub app_type: AppType,
    /// Selected Nest crates to include.
    pub selected_crates: Vec<String>,
}

/// Response from scaffolding operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldResponse {
    /// Whether the operation succeeded.
    pub success: bool,
    /// Human-readable message (success details or error).
    pub message: String,
    /// Path to the created application (relative to nest root).
    pub app_path: Option<String>,
}

/// Get the list of available core crates from the active workspace.
///
/// Everything the wizard touches — templates, `core/crates`, and the `apps/`
/// directory the new app lands in — is anchored to the active Kiwi workspace
/// root (resolved from `KIWI_PROJECT_ROOT` / config `[project].root`), NOT to
/// Kiwi's own source tree. This is why a workspace pointed at `/data/projects/x`
/// scaffolds into `/data/projects/x/apps`.
#[tauri::command]
pub fn new_app_list_crates(workspace: State<'_, Workspace>) -> NestResult<Vec<String>> {
    list_core_crates(&workspace.root())
}

/// List the `nest-*` crates available under `<root>/core/crates`.
fn list_core_crates(root: &Path) -> NestResult<Vec<String>> {
    let core_crates_dir = root.join("core/crates");
    if !core_crates_dir.exists() {
        return Err(NestError::io("core/crates directory not found"));
    }

    let mut crates = Vec::new();
    for entry in fs::read_dir(&core_crates_dir)
        .map_err(|e| NestError::io(format!("failed to read core/crates: {e}")))?
    {
        let entry = entry.map_err(|e| NestError::io(format!("failed to read entry: {e}")))?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name != "logs" && name.starts_with("nest-") {
                    crates.push(name.to_string());
                }
            }
        }
    }

    crates.sort();
    Ok(crates)
}

/// Per-app-type crate selection: `required` are structural (always included,
/// shown locked in the wizard), `recommended` are checked by default, and
/// `optional` is every other core crate.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrateProfile {
    pub required: Vec<String>,
    pub recommended: Vec<String>,
    pub optional: Vec<String>,
}

/// The `(required, recommended)` crate lists for an app type. `required` mirrors
/// the crates each scaffolder wires structurally; keep the two in sync.
fn crate_profile_lists(app_type: &AppType) -> (Vec<&'static str>, Vec<&'static str>) {
    // `required` MUST match the crates each scaffolder wires structurally (see the
    // matching scaffold_* function), so the wizard's locked list is truthful.
    match app_type {
        AppType::Gui => (
            vec!["nest-tauri", "nest-theme", "nest-image", "nest-cache", "nest-error"],
            vec!["nest-icon", "nest-config", "nest-logging"],
        ),
        AppType::Tui => (
            vec!["nest-tui", "nest-error"],
            vec!["nest-app", "nest-config", "nest-logging"],
        ),
        AppType::Cli => (
            vec!["nest-cli", "nest-app", "nest-error"],
            vec!["nest-config", "nest-logging", "nest-file"],
        ),
        AppType::System => (
            vec!["nest-app", "nest-config", "nest-logging", "nest-error"],
            vec!["nest-task", "nest-task-runtime"],
        ),
        AppType::ApiServer => (
            vec!["nest-http-serve", "nest-error"],
            vec!["nest-app", "nest-config", "nest-logging", "nest-validation"],
        ),
        AppType::ApiServerWeb => (
            vec!["nest-http-serve", "nest-error"],
            vec!["nest-app", "nest-config", "nest-logging", "nest-http-client", "nest-validation"],
        ),
    }
}

/// Get the crate profile for an app type: required/recommended from the static
/// table, and optional = every other core crate in the workspace.
#[tauri::command]
pub fn new_app_crate_profile(
    workspace: State<'_, Workspace>,
    app_type: AppType,
) -> NestResult<CrateProfile> {
    let (required, recommended) = crate_profile_lists(&app_type);
    let required: Vec<String> = required.iter().map(|s| s.to_string()).collect();
    let recommended: Vec<String> = recommended.iter().map(|s| s.to_string()).collect();

    let optional = list_core_crates(&workspace.root())
        .unwrap_or_default()
        .into_iter()
        .filter(|c| !required.contains(c) && !recommended.contains(c))
        .collect();

    Ok(CrateProfile {
        required,
        recommended,
        optional,
    })
}

/// Event channel the wizard listens on for live scaffolding progress.
pub const NEW_APP_PROGRESS_EVENT: &str = "new-app://progress";

/// A single progress line emitted while scaffolding.
#[derive(Debug, Clone, Serialize)]
pub struct NewAppProgress {
    /// Human-readable step description.
    pub message: String,
    /// Whether this line represents a failure.
    pub error: bool,
}

/// Core scaffolding routine shared by the commands. `progress` is called for
/// each step so the UI can show where work is happening — or where it fails.
/// Returns the created app path relative to the monorepo root.
fn run_scaffold(
    root: &Path,
    request: &ScaffoldRequest,
    progress: &dyn Fn(&str),
) -> NestResult<String> {
    let name = request.name.trim();
    progress(&format!("Validating name '{name}'"));
    if name.is_empty() {
        return Err(NestError::validation("Application name cannot be empty"));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(NestError::validation(
            "Application name must contain only alphanumeric characters, hyphens, and underscores",
        ));
    }

    progress(&format!("Workspace root: {}", root.display()));
    let apps_dir = root.join("apps");
    if !apps_dir.exists() {
        return Err(NestError::io(format!(
            "no 'apps' directory in the active workspace ({})",
            root.display()
        )));
    }
    let app_dir = apps_dir.join(name);

    if app_dir.exists() {
        return Err(NestError::validation(format!(
            "Application '{}' already exists at {}",
            name,
            app_dir.display()
        )));
    }

    progress(&format!("Creating apps/{name}"));
    fs::create_dir_all(&app_dir)
        .map_err(|e| NestError::io(format!("failed to create app directory: {e}")))?;

    let result = match request.app_type {
        AppType::Gui => scaffold_gui_app(root, &app_dir, name, &request.selected_crates, progress),
        AppType::Tui => scaffold_tui_app(&app_dir, name, &request.selected_crates, progress),
        AppType::Cli => scaffold_cli_app(&app_dir, name, &request.selected_crates, progress),
        AppType::System => scaffold_system_app(&app_dir, name, &request.selected_crates, progress),
        AppType::ApiServer => {
            scaffold_api_app(&app_dir, name, &request.selected_crates, false, progress)
        }
        AppType::ApiServerWeb => {
            scaffold_api_app(&app_dir, name, &request.selected_crates, true, progress)
        }
    };

    match result {
        Ok(()) => Ok(app_dir
            .strip_prefix(root)
            .unwrap_or(&app_dir)
            .to_string_lossy()
            .to_string()),
        Err(e) => {
            // Remove a partial scaffold so retrying with the same name works.
            let _ = fs::remove_dir_all(&app_dir);
            Err(e)
        }
    }
}

/// Emit a progress line to the wizard, ignoring transport errors.
fn emit_progress<R: Runtime>(app: &AppHandle<R>, message: &str, error: bool) {
    let _ = app.emit(
        NEW_APP_PROGRESS_EVENT,
        NewAppProgress {
            message: message.to_string(),
            error,
        },
    );
}

/// Scaffold a new application, streaming progress to the wizard. This only
/// writes files (fast); it does not compile anything.
#[tauri::command]
pub fn new_app_scaffold<R: Runtime>(
    app: AppHandle<R>,
    workspace: State<'_, Workspace>,
    request: ScaffoldRequest,
) -> NestResult<ScaffoldResponse> {
    let emit = |msg: &str| emit_progress(&app, msg, false);
    match run_scaffold(&workspace.root(), &request, &emit) {
        Ok(path) => {
            emit(&format!("Created {path}"));
            Ok(ScaffoldResponse {
                success: true,
                message: format!("Successfully created '{}'", request.name.trim()),
                app_path: Some(path),
            })
        }
        Err(e) => {
            emit_progress(&app, &format!("Error: {e}"), true);
            Err(e)
        }
    }
}

/// Scaffold and then verify the app with `cargo check`. Not used by the wizard's
/// Create button — a GUI check compiles the whole Tauri stack and takes minutes —
/// but exposed for an explicit, opt-in verification flow.
#[tauri::command]
pub fn new_app_build<R: Runtime>(
    app: AppHandle<R>,
    workspace: State<'_, Workspace>,
    request: ScaffoldRequest,
) -> NestResult<ScaffoldResponse> {
    let emit = |msg: &str| emit_progress(&app, msg, false);
    let root = workspace.root();
    let path = match run_scaffold(&root, &request, &emit) {
        Ok(path) => path,
        Err(e) => {
            emit_progress(&app, &format!("Error: {e}"), true);
            return Err(e);
        }
    };

    emit("Verifying build with cargo check (this can take a while)");
    let app_dir = root.join(&path);
    match run_cargo_check(&app_dir) {
        Ok(()) => {
            emit("Build verified");
            Ok(ScaffoldResponse {
                success: true,
                message: format!("Created and verified '{}'", request.name.trim()),
                app_path: Some(path),
            })
        }
        Err(e) => {
            emit_progress(&app, &format!("Build verification failed: {e}"), true);
            Ok(ScaffoldResponse {
                success: false,
                message: format!("App created but build verification failed: {e}"),
                app_path: Some(path),
            })
        }
    }
}

fn run_cargo_check(app_dir: &Path) -> NestResult<()> {
    // Look for Cargo.toml to determine the project structure
    let cargo_toml = if app_dir.join("src-tauri/Cargo.toml").exists() {
        app_dir.join("src-tauri/Cargo.toml")
    } else if app_dir.join("crates/cli/Cargo.toml").exists() {
        app_dir.join("crates/cli/Cargo.toml")
    } else if app_dir.join("Cargo.toml").exists() {
        app_dir.join("Cargo.toml")
    } else {
        return Err(NestError::io("No Cargo.toml found in scaffolded app"));
    };

    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(&cargo_toml)
        .output()
        .map_err(|e| NestError::io(format!("failed to run cargo check: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(NestError::io(format!("cargo check failed: {}", stderr)));
    }

    Ok(())
}

/// Merge a fixed set of required crates with the user-selected crates,
/// preserving order and dropping duplicates.
fn merge_crates(required: &[&str], selected: &[String]) -> Vec<String> {
    let mut out: Vec<String> = required.iter().map(|s| s.to_string()).collect();
    for c in selected {
        if !out.iter().any(|existing| existing == c) {
            out.push(c.clone());
        }
    }
    out
}

/// Render `name = { path = "<prefix>/name" }` lines, one per crate.
fn crate_path_deps(crates: &[String], prefix: &str) -> String {
    crates
        .iter()
        .map(|c| format!("{c} = {{ path = \"{prefix}/{c}\" }}\n"))
        .collect()
}

/// Render `name = { workspace = true }` lines, one per crate.
fn crate_workspace_deps(crates: &[String]) -> String {
    crates
        .iter()
        .map(|c| format!("{c} = {{ workspace = true }}\n"))
        .collect()
}

fn scaffold_gui_app(
    root: &Path,
    app_dir: &Path,
    name: &str,
    crates: &[String],
    progress: &dyn Fn(&str),
) -> NestResult<()> {
    let template_dir = root.join("templates/desktop");
    if !template_dir.exists() {
        return Err(NestError::io(format!(
            "desktop template not found at {}",
            template_dir.display()
        )));
    }

    // Copy template files
    progress("Copying template: ui");
    copy_template_dir(&template_dir.join("ui"), &app_dir.join("ui"))?;
    progress("Copying template: src-tauri");
    copy_template_dir(&template_dir.join("src-tauri"), &app_dir.join("src-tauri"))?;
    copy_template_file(&template_dir.join("build"), &app_dir.join("build"))?;
    copy_template_file(&template_dir.join(".gitignore"), &app_dir.join(".gitignore"))?;

    // Update tauri.conf.json with app name
    let tauri_conf_path = app_dir.join("src-tauri/tauri.conf.json");
    if let Ok(mut content) = fs::read_to_string(&tauri_conf_path) {
        content = content
            .replace("Nest Desktop Template", &title_case(name))
            .replace("com.nest.desktop-template", &format!("com.nest.{}", name.replace('_', "-")));
        fs::write(&tauri_conf_path, content)
            .map_err(|e| NestError::io(format!("failed to update tauri.conf.json: {e}")))?;
    }

    // Update main.rs with app name
    let main_rs_path = app_dir.join("src-tauri/src/main.rs");
    if let Ok(mut content) = fs::read_to_string(&main_rs_path) {
        content = content.replace("nest-desktop-template", &name.replace('_', "-"));
        fs::write(&main_rs_path, content)
            .map_err(|e| NestError::io(format!("failed to update main.rs: {e}")))?;
    }

    // Create .cargo/config.toml with path patches
    let cargo_config_dir = app_dir.join(".cargo");
    fs::create_dir_all(&cargo_config_dir)
        .map_err(|e| NestError::io(format!("failed to create .cargo dir: {e}")))?;

    let cargo_config = r#"[patch.crates-io]
# Path patches for local development with Nest monorepo
nest-app = { path = "../../core/crates/nest-app" }
nest-cache = { path = "../../core/crates/nest-cache" }
nest-config = { path = "../../core/crates/nest-config" }
nest-error = { path = "../../core/crates/nest-error" }
nest-http = { path = "../../core/crates/nest-http" }
nest-http-client = { path = "../../core/crates/nest-http-client" }
nest-image = { path = "../../core/crates/nest-image" }
nest-logging = { path = "../../core/crates/nest-logging" }
nest-tauri = { path = "../../core/crates/nest-tauri" }
nest-theme = { path = "../../core/crates/nest-theme" }
nest-validation = { path = "../../core/crates/nest-validation" }
"#;

    fs::write(cargo_config_dir.join("config.toml"), cargo_config)
        .map_err(|e| NestError::io(format!("failed to write .cargo/config.toml: {e}")))?;

    // Create config.example.toml
    let config_example = format!(
        r#"[tauri]
title = "{}"
width = 1400
height = 900
"#,
        title_case(name)
    );

    fs::write(app_dir.join("config.example.toml"), config_example)
        .map_err(|e| NestError::io(format!("failed to write config.example.toml: {e}")))?;

    progress("Creating crates/core");
    // Create crates/core, the app-specific logic crate. It is a standalone crate
    // (not a workspace member) that src-tauri depends on by path, so it is built
    // and verified as part of the app's `cargo check`. The user-selected Nest
    // crates are wired in here as explicit path dependencies.
    let kebab = name.replace('_', "-");
    let core_crate_dir = app_dir.join("crates/core");
    fs::create_dir_all(core_crate_dir.join("src"))
        .map_err(|e| NestError::io(format!("failed to create crates/core: {e}")))?;

    // Create a placeholder lib.rs
    let lib_rs = format!(
        r#"//! {name} core module

pub fn greet(name: &str) -> String {{
    format!("Hello, {{name}}!")
}}
"#
    );

    fs::write(core_crate_dir.join("src/lib.rs"), lib_rs)
        .map_err(|e| NestError::io(format!("failed to write lib.rs: {e}")))?;

    // crates/core lives at apps/<name>/crates/core, four levels below the repo
    // root, so path dependencies into core/crates are prefixed accordingly.
    let core_deps = crate_path_deps(
        &merge_crates(&["nest-error"], crates),
        "../../../../core/crates",
    );
    // The trailing empty `[workspace]` makes crates/core its own workspace root.
    // Without it, cargo walks up and attaches this crate to the repo-root
    // workspace (the GUI app has no app-level workspace to shield it).
    let core_cargo = format!(
        r#"[package]
name = "{kebab}-core"
version = "0.1.0"
edition = "2021"

[dependencies]
{core_deps}
[workspace]
"#
    );

    fs::write(core_crate_dir.join("Cargo.toml"), core_cargo)
        .map_err(|e| NestError::io(format!("failed to write core Cargo.toml: {e}")))?;

    progress("Wiring crates/core into src-tauri");
    // Wire crates/core into the src-tauri crate so it is compiled and verified.
    let src_tauri_cargo = app_dir.join("src-tauri/Cargo.toml");
    let content = fs::read_to_string(&src_tauri_cargo)
        .map_err(|e| NestError::io(format!("failed to read src-tauri Cargo.toml: {e}")))?;
    let content = content.replacen(
        "[dependencies]\n",
        &format!("[dependencies]\n{kebab}-core = {{ path = \"../crates/core\" }}\n"),
        1,
    );
    fs::write(&src_tauri_cargo, content)
        .map_err(|e| NestError::io(format!("failed to update src-tauri Cargo.toml: {e}")))?;

    // Create README.md
    let readme = format!(
        r#"# {}

Generated Nest desktop application.

## Quick Start

```bash
./build dev      # Run in development mode
./build run      # Build and run
./build build    # Production build
```

## Project Structure

- `ui/` - React + TypeScript + Tailwind frontend
- `src-tauri/` - Tauri backend
- `crates/core/` - Application-specific Rust logic
"#,
        title_case(name)
    );

    fs::write(app_dir.join("README.md"), readme)
        .map_err(|e| NestError::io(format!("failed to write README.md: {e}")))?;

    // Set execute permissions on build script (Unix only)
    let build_path = app_dir.join("build");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&build_path, fs::Permissions::from_mode(0o755));
    }

    Ok(())
}

/// `main.rs` for a scaffolded CLI app. `__CORE_CRATE__` is replaced with the
/// app's core crate name so the example command can call into it.
const CLI_MAIN_TEMPLATE: &str = r#"//! CLI entry point.

use nest_cli::{AppContext, CliApp, CliCommand, NestResult};

/// Example subcommand. Replace with your own commands.
struct HelloCommand;

impl CliCommand for HelloCommand {
    fn name(&self) -> &'static str {
        "hello"
    }

    fn about(&self) -> &'static str {
        "Print a friendly greeting"
    }

    fn configure(&self, cmd: clap::Command) -> clap::Command {
        cmd.arg(
            clap::Arg::new("name")
                .help("Who to greet")
                .default_value("world"),
        )
    }

    fn run(&self, _ctx: &AppContext, matches: &clap::ArgMatches) -> NestResult<()> {
        let who = matches
            .get_one::<String>("name")
            .map(String::as_str)
            .unwrap_or("world");
        println!("{}", __CORE_CRATE__::greet(who));
        Ok(())
    }
}

fn main() {
    CliApp::new(env!("CARGO_PKG_NAME"))
        .with_version(env!("CARGO_PKG_VERSION"))
        .command(HelloCommand)
        .run();
}
"#;

/// `main.rs` for a scaffolded System (daemon/service) app: builds a `NestApp`,
/// starts it, and runs a service loop until the process is stopped.
const SERVICE_MAIN_TEMPLATE: &str = r#"//! Service entry point.

use std::thread;
use std::time::Duration;

use nest_app::{NestApp, NestResult};

fn main() {
    if let Err(err) = run() {
        eprintln!("fatal: {err}");
        std::process::exit(1);
    }
}

fn run() -> NestResult<()> {
    let mut app = NestApp::builder(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .build()?;
    app.startup()?;

    println!("{} service started (Ctrl-C to stop)", env!("CARGO_PKG_NAME"));

    // Main service loop. Replace the body with your real work: poll a queue,
    // serve requests, run scheduled jobs, and so on.
    loop {
        thread::sleep(Duration::from_secs(5));
    }
}
"#;

/// Write the shared `./build` helper script (used by CLI and System apps) and
/// mark it executable.
fn write_build_script(app_dir: &Path) -> NestResult<()> {
    let build_script = r#"#!/usr/bin/env bash
set -euo pipefail

APP_ROOT="$(cd "$(dirname "$0")" && pwd)"
export CARGO_TARGET_DIR="$APP_ROOT/target"

cd "$APP_ROOT"

cmd="${1:-run}"
shift || true

case "$cmd" in
  build)
    cargo build --workspace "$@"
    ;;
  release)
    cargo build --workspace --release "$@"
    ;;
  run)
    # `cargo run` takes no --workspace; target the app's binary by name (the
    # bin is named after the app, hyphenated).
    cargo run --bin "$(basename "$APP_ROOT" | tr '_' '-')" "$@"
    ;;
  test)
    cargo test --workspace "$@"
    ;;
  check)
    cargo fmt --all --check
    cargo clippy --workspace -- -D warnings
    ;;
  clean)
    cargo clean
    ;;
  *)
    cargo "$cmd" "$@"
    ;;
esac
"#;

    let build_path = app_dir.join("build");
    fs::write(&build_path, build_script)
        .map_err(|e| NestError::io(format!("failed to write build script: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&build_path, fs::Permissions::from_mode(0o755))
            .map_err(|e| NestError::io(format!("failed to set build permissions: {e}")))?;
    }

    Ok(())
}

fn scaffold_cli_app(
    app_dir: &Path,
    name: &str,
    crates: &[String],
    progress: &dyn Fn(&str),
) -> NestResult<()> {
    progress("Writing CLI workspace");
    let snake = name.replace('-', "_");
    let kebab = name.replace('_', "-");
    let core_crate = format!("{snake}_core");

    let crates_dir = app_dir.join("crates");
    let core_dir = crates_dir.join("core");
    let cli_dir = crates_dir.join("cli");

    fs::create_dir_all(core_dir.join("src"))
        .map_err(|e| NestError::io(format!("failed to create crates/core: {e}")))?;
    fs::create_dir_all(cli_dir.join("src"))
        .map_err(|e| NestError::io(format!("failed to create crates/cli: {e}")))?;

    // Workspace root. Selected crates are declared here and consumed by the
    // member crates via `{ workspace = true }`.
    let workspace_nest = merge_crates(&["nest-error", "nest-app", "nest-cli"], crates);
    let workspace_deps = crate_path_deps(&workspace_nest, "../../core/crates");
    let workspace_cargo = format!(
        r#"[workspace]
members = ["crates/core", "crates/cli"]
resolver = "2"

[workspace.dependencies]
{workspace_deps}clap = {{ version = "4", features = ["derive", "env"] }}

[profile.release]
lto = true
"#
    );

    fs::write(app_dir.join("Cargo.toml"), workspace_cargo)
        .map_err(|e| NestError::io(format!("failed to write workspace Cargo.toml: {e}")))?;

    // Core library crate: app logic plus the selected Nest crates.
    let core_deps = crate_workspace_deps(&merge_crates(&["nest-error"], crates));
    let core_cargo = format!(
        r#"[package]
name = "{kebab}-core"
version = "0.1.0"
edition = "2021"

[dependencies]
{core_deps}"#
    );

    fs::write(core_dir.join("Cargo.toml"), core_cargo)
        .map_err(|e| NestError::io(format!("failed to write core Cargo.toml: {e}")))?;

    let core_lib = format!(
        r#"//! {name} core logic.

/// Build a greeting message.
pub fn greet(name: &str) -> String {{
    format!("Hello, {{name}}!")
}}
"#
    );

    fs::write(core_dir.join("src/lib.rs"), core_lib)
        .map_err(|e| NestError::io(format!("failed to write core lib.rs: {e}")))?;

    // Binary crate.
    let cli_bin_deps =
        crate_workspace_deps(&["nest-app".to_string(), "nest-cli".to_string()]);
    let cli_cargo = format!(
        r#"[package]
name = "{kebab}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{kebab}"
path = "src/main.rs"

[dependencies]
{kebab}-core = {{ path = "../core" }}
{cli_bin_deps}clap = {{ workspace = true }}
"#
    );

    fs::write(cli_dir.join("Cargo.toml"), cli_cargo)
        .map_err(|e| NestError::io(format!("failed to write cli Cargo.toml: {e}")))?;

    let cli_main = CLI_MAIN_TEMPLATE.replace("__CORE_CRATE__", &core_crate);
    fs::write(cli_dir.join("src/main.rs"), cli_main)
        .map_err(|e| NestError::io(format!("failed to write cli main.rs: {e}")))?;

    progress("Writing build script");
    write_build_script(app_dir)?;

    let config_example = format!(
        r#"[app]
name = "{}"
"#,
        title_case(name)
    );

    fs::write(app_dir.join("config.example.toml"), config_example)
        .map_err(|e| NestError::io(format!("failed to write config.example.toml: {e}")))?;

    let readme = format!(
        r#"# {}

Generated Nest CLI application.

## Quick Start

```bash
./build run      # Run the CLI (try: ./build run -- hello Ada)
./build build    # Build release
./build check    # Run linting
```
"#,
        title_case(name)
    );

    fs::write(app_dir.join("README.md"), readme)
        .map_err(|e| NestError::io(format!("failed to write README.md: {e}")))?;

    Ok(())
}

fn scaffold_system_app(
    app_dir: &Path,
    name: &str,
    crates: &[String],
    progress: &dyn Fn(&str),
) -> NestResult<()> {
    progress("Writing service workspace");
    let kebab = name.replace('_', "-");

    let crates_dir = app_dir.join("crates");
    let core_dir = crates_dir.join("core");
    let service_dir = crates_dir.join("service");

    fs::create_dir_all(core_dir.join("src"))
        .map_err(|e| NestError::io(format!("failed to create crates/core: {e}")))?;
    fs::create_dir_all(service_dir.join("src"))
        .map_err(|e| NestError::io(format!("failed to create crates/service: {e}")))?;

    // Workspace root.
    let workspace_nest = merge_crates(
        &["nest-error", "nest-app", "nest-config", "nest-logging"],
        crates,
    );
    let workspace_deps = crate_path_deps(&workspace_nest, "../../core/crates");
    let workspace_cargo = format!(
        r#"[workspace]
members = ["crates/core", "crates/service"]
resolver = "2"

[workspace.dependencies]
{workspace_deps}
[profile.release]
lto = true
"#
    );

    fs::write(app_dir.join("Cargo.toml"), workspace_cargo)
        .map_err(|e| NestError::io(format!("failed to write workspace Cargo.toml: {e}")))?;

    // Core library crate: shared logic plus the selected Nest crates.
    let core_deps = crate_workspace_deps(&merge_crates(&["nest-error"], crates));
    let core_cargo = format!(
        r#"[package]
name = "{kebab}-core"
version = "0.1.0"
edition = "2021"

[dependencies]
{core_deps}"#
    );

    fs::write(core_dir.join("Cargo.toml"), core_cargo)
        .map_err(|e| NestError::io(format!("failed to write core Cargo.toml: {e}")))?;

    let core_lib = format!(
        r#"//! {name} core logic.

/// Placeholder for shared service logic.
pub fn describe() -> &'static str {{
    "{name} service"
}}
"#
    );

    fs::write(core_dir.join("src/lib.rs"), core_lib)
        .map_err(|e| NestError::io(format!("failed to write core lib.rs: {e}")))?;

    // Service binary crate.
    let service_bin_deps = crate_workspace_deps(&[
        "nest-app".to_string(),
        "nest-config".to_string(),
        "nest-logging".to_string(),
    ]);
    let service_cargo = format!(
        r#"[package]
name = "{kebab}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{kebab}"
path = "src/main.rs"

[dependencies]
{kebab}-core = {{ path = "../core" }}
{service_bin_deps}"#
    );

    fs::write(service_dir.join("Cargo.toml"), service_cargo)
        .map_err(|e| NestError::io(format!("failed to write service Cargo.toml: {e}")))?;

    fs::write(service_dir.join("src/main.rs"), SERVICE_MAIN_TEMPLATE)
        .map_err(|e| NestError::io(format!("failed to write service main.rs: {e}")))?;

    progress("Writing build script");
    write_build_script(app_dir)?;

    let config_example = format!(
        r#"[app]
name = "{}"
"#,
        title_case(name)
    );

    fs::write(app_dir.join("config.example.toml"), config_example)
        .map_err(|e| NestError::io(format!("failed to write config.example.toml: {e}")))?;

    let readme = format!(
        r#"# {}

Generated Nest system service.

## Quick Start

```bash
./build run      # Run the service (Ctrl-C to stop)
./build build    # Build release
./build check    # Run linting
```
"#,
        title_case(name)
    );

    fs::write(app_dir.join("README.md"), readme)
        .map_err(|e| NestError::io(format!("failed to write README.md: {e}")))?;

    Ok(())
}

/// `main.rs` for a scaffolded TUI app. `__CORE_CRATE__` → the app's core crate.
const TUI_MAIN_TEMPLATE: &str = r#"//! Terminal UI entry point.

use crossterm::event::{Event, KeyCode};
use nest_tui::{AppContext, NestResult, TuiAction, TuiApp, TuiScreen};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Root screen. Replace with your own UI.
struct HomeScreen;

impl TuiScreen for HomeScreen {
    fn draw(&mut self, frame: &mut Frame, _ctx: &AppContext) -> NestResult<()> {
        let body = Paragraph::new(__CORE_CRATE__::greeting())
            .block(Block::default().title(env!("CARGO_PKG_NAME")).borders(Borders::ALL));
        frame.render_widget(body, frame.area());
        Ok(())
    }

    fn on_event(&mut self, event: Event, _ctx: &AppContext) -> NestResult<TuiAction> {
        if let Event::Key(key) = event {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                return Ok(TuiAction::Quit);
            }
        }
        Ok(TuiAction::Continue)
    }
}

fn main() {
    TuiApp::new(env!("CARGO_PKG_NAME"))
        .screen(HomeScreen)
        .run();
}
"#;

/// `main.rs` for a scaffolded API server. `__CORE_CRATE__` → the app's core crate.
const API_MAIN_TEMPLATE: &str = r#"//! HTTP API server entry point.

use nest_http_serve::{HttpResult, HttpServer, Json, RequestContext, RouteGroup};
use serde::Serialize;

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

#[derive(Serialize)]
struct Greeting {
    message: String,
}

async fn health(_ctx: RequestContext) -> HttpResult {
    Json(Health { status: "ok" }).into_response()
}

async fn hello(_ctx: RequestContext) -> HttpResult {
    Json(Greeting {
        message: __CORE_CRATE__::greeting(),
    })
    .into_response()
}

#[tokio::main]
async fn main() {
    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    println!("{} listening on http://{addr}", env!("CARGO_PKG_NAME"));

    if let Err(err) = HttpServer::builder()
        .bind(&addr)
        .routes(
            RouteGroup::new("/api")
                .get("/health", health)
                .get("/hello", hello),
        )
        .run()
        .await
    {
        eprintln!("server error: {err:?}");
        std::process::exit(1);
    }
}
"#;

/// Write the shared `crates/core` library (a `greet`-style helper) used by the
/// workspace-based app types.
fn write_core_crate(core_dir: &Path, kebab: &str, name: &str, crates: &[String]) -> NestResult<()> {
    let core_deps = crate_workspace_deps(&merge_crates(&["nest-error"], crates));
    let core_cargo = format!(
        r#"[package]
name = "{kebab}-core"
version = "0.1.0"
edition = "2021"

[dependencies]
{core_deps}"#
    );
    fs::write(core_dir.join("Cargo.toml"), core_cargo)
        .map_err(|e| NestError::io(format!("failed to write core Cargo.toml: {e}")))?;

    let core_lib = format!(
        r#"//! {name} core logic.

/// A greeting used by the app's entry point.
pub fn greeting() -> String {{
    "Hello from {name}!".to_string()
}}
"#
    );
    fs::write(core_dir.join("src/lib.rs"), core_lib)
        .map_err(|e| NestError::io(format!("failed to write core lib.rs: {e}")))?;
    Ok(())
}

fn scaffold_tui_app(
    app_dir: &Path,
    name: &str,
    crates: &[String],
    progress: &dyn Fn(&str),
) -> NestResult<()> {
    progress("Writing TUI workspace");
    let snake = name.replace('-', "_");
    let kebab = name.replace('_', "-");
    let core_crate = format!("{snake}_core");

    let core_dir = app_dir.join("crates/core");
    let tui_dir = app_dir.join("crates/tui");
    fs::create_dir_all(core_dir.join("src"))
        .map_err(|e| NestError::io(format!("failed to create crates/core: {e}")))?;
    fs::create_dir_all(tui_dir.join("src"))
        .map_err(|e| NestError::io(format!("failed to create crates/tui: {e}")))?;

    // Structural deps: core uses nest-error, the tui bin uses nest-tui. Anything
    // the user selected (recommended/optional) is wired into crates/core below.
    let workspace_nest = merge_crates(&["nest-error", "nest-tui"], crates);
    let workspace_deps = crate_path_deps(&workspace_nest, "../../core/crates");
    let workspace_cargo = format!(
        r#"[workspace]
members = ["crates/core", "crates/tui"]
resolver = "2"

[workspace.dependencies]
{workspace_deps}ratatui = "0.29"
crossterm = "0.28"

[profile.release]
lto = true
"#
    );
    fs::write(app_dir.join("Cargo.toml"), workspace_cargo)
        .map_err(|e| NestError::io(format!("failed to write workspace Cargo.toml: {e}")))?;

    write_core_crate(&core_dir, &kebab, name, crates)?;

    let tui_bin_deps = crate_workspace_deps(&["nest-tui".to_string()]);
    let tui_cargo = format!(
        r#"[package]
name = "{kebab}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{kebab}"
path = "src/main.rs"

[dependencies]
{kebab}-core = {{ path = "../core" }}
{tui_bin_deps}ratatui = {{ workspace = true }}
crossterm = {{ workspace = true }}
"#
    );
    fs::write(tui_dir.join("Cargo.toml"), tui_cargo)
        .map_err(|e| NestError::io(format!("failed to write tui Cargo.toml: {e}")))?;

    let tui_main = TUI_MAIN_TEMPLATE.replace("__CORE_CRATE__", &core_crate);
    fs::write(tui_dir.join("src/main.rs"), tui_main)
        .map_err(|e| NestError::io(format!("failed to write tui main.rs: {e}")))?;

    progress("Writing build script");
    write_build_script(app_dir)?;
    write_config_and_readme(app_dir, name, "Nest terminal UI application", "./build run      # Run the TUI (press q to quit)")?;
    Ok(())
}

fn scaffold_api_app(
    app_dir: &Path,
    name: &str,
    crates: &[String],
    with_web: bool,
    progress: &dyn Fn(&str),
) -> NestResult<()> {
    progress("Writing API server workspace");
    let snake = name.replace('-', "_");
    let kebab = name.replace('_', "-");
    let core_crate = format!("{snake}_core");

    let core_dir = app_dir.join("crates/core");
    let server_dir = app_dir.join("crates/server");
    fs::create_dir_all(core_dir.join("src"))
        .map_err(|e| NestError::io(format!("failed to create crates/core: {e}")))?;
    fs::create_dir_all(server_dir.join("src"))
        .map_err(|e| NestError::io(format!("failed to create crates/server: {e}")))?;

    // Structural deps: core uses nest-error, the server bin uses nest-http-serve.
    // Selected crates (recommended/optional, incl. nest-http-client for the web
    // variant) are wired into crates/core below. `with_web` only adds a frontend.
    let workspace_nest = merge_crates(&["nest-error", "nest-http-serve"], crates);
    let workspace_deps = crate_path_deps(&workspace_nest, "../../core/crates");
    let workspace_cargo = format!(
        r#"[workspace]
members = ["crates/core", "crates/server"]
resolver = "2"

[workspace.dependencies]
{workspace_deps}tokio = {{ version = "1", features = ["macros", "rt-multi-thread"] }}
serde = {{ version = "1", features = ["derive"] }}

[profile.release]
lto = true
"#
    );
    fs::write(app_dir.join("Cargo.toml"), workspace_cargo)
        .map_err(|e| NestError::io(format!("failed to write workspace Cargo.toml: {e}")))?;

    write_core_crate(&core_dir, &kebab, name, crates)?;

    let server_bin_deps = crate_workspace_deps(&["nest-http-serve".to_string()]);
    let server_cargo = format!(
        r#"[package]
name = "{kebab}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{kebab}"
path = "src/main.rs"

[dependencies]
{kebab}-core = {{ path = "../core" }}
{server_bin_deps}tokio = {{ workspace = true }}
serde = {{ workspace = true }}
"#
    );
    fs::write(server_dir.join("Cargo.toml"), server_cargo)
        .map_err(|e| NestError::io(format!("failed to write server Cargo.toml: {e}")))?;

    let server_main = API_MAIN_TEMPLATE.replace("__CORE_CRATE__", &core_crate);
    fs::write(server_dir.join("src/main.rs"), server_main)
        .map_err(|e| NestError::io(format!("failed to write server main.rs: {e}")))?;

    if with_web {
        progress("Writing web front end (Vite + React)");
        write_api_web_frontend(app_dir, name)?;
    }

    progress("Writing build script");
    write_build_script(app_dir)?;
    let readme_run = if with_web {
        "./build run      # Run the API server (then `cd web && npm install && npm run dev`)"
    } else {
        "./build run      # Run the API server (GET http://localhost:3000/api/health)"
    };
    write_config_and_readme(app_dir, name, "Nest HTTP API server", readme_run)?;
    Ok(())
}

/// Shared config.example.toml + README writer for the workspace-based app types.
fn write_config_and_readme(
    app_dir: &Path,
    name: &str,
    summary: &str,
    run_line: &str,
) -> NestResult<()> {
    let config_example = format!("[app]\nname = \"{}\"\n", title_case(name));
    fs::write(app_dir.join("config.example.toml"), config_example)
        .map_err(|e| NestError::io(format!("failed to write config.example.toml: {e}")))?;

    let readme = format!(
        r#"# {title}

{summary}.

## Quick Start

```bash
{run_line}
./build build    # Build release
./build check    # Run linting
```
"#,
        title = title_case(name),
    );
    fs::write(app_dir.join("README.md"), readme)
        .map_err(|e| NestError::io(format!("failed to write README.md: {e}")))?;
    Ok(())
}

/// Write a minimal Vite + React front end under `web/` for the API+Web type.
/// It proxies `/api` to the Rust server and shows the `/api/health` status.
fn write_api_web_frontend(app_dir: &Path, name: &str) -> NestResult<()> {
    let kebab = name.replace('_', "-");
    let title = title_case(name);
    let web = app_dir.join("web");
    fs::create_dir_all(web.join("src"))
        .map_err(|e| NestError::io(format!("failed to create web dir: {e}")))?;

    let files: [(&str, &str); 6] = [
        ("package.json", WEB_PACKAGE_JSON),
        ("index.html", WEB_INDEX_HTML),
        ("vite.config.ts", WEB_VITE_CONFIG),
        ("tsconfig.json", WEB_TSCONFIG),
        ("src/main.tsx", WEB_MAIN_TSX),
        ("src/App.tsx", WEB_APP_TSX),
    ];
    for (rel, content) in files {
        let rendered = content.replace("__KEBAB__", &kebab).replace("__TITLE__", &title);
        fs::write(web.join(rel), rendered)
            .map_err(|e| NestError::io(format!("failed to write web/{rel}: {e}")))?;
    }
    Ok(())
}

const WEB_PACKAGE_JSON: &str = r#"{
  "name": "__KEBAB__-web",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "react": "^19.1.0",
    "react-dom": "^19.1.0"
  },
  "devDependencies": {
    "@types/react": "^19.1.0",
    "@types/react-dom": "^19.1.0",
    "@vitejs/plugin-react": "^4.4.0",
    "typescript": "^5.8.3",
    "vite": "^6.3.0"
  }
}
"#;

const WEB_INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>__TITLE__</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#;

const WEB_VITE_CONFIG: &str = r#"import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Proxy /api to the Rust server during development.
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/api": "http://localhost:3000",
    },
  },
});
"#;

const WEB_TSCONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true
  },
  "include": ["src"]
}
"#;

const WEB_MAIN_TSX: &str = r#"import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
"#;

const WEB_APP_TSX: &str = r#"import { useEffect, useState } from "react";

export function App() {
  const [status, setStatus] = useState("loading…");

  useEffect(() => {
    fetch("/api/health")
      .then((res) => res.json())
      .then((data) => setStatus(String(data.status)))
      .catch(() => setStatus("unreachable"));
  }, []);

  return (
    <main style={{ fontFamily: "system-ui", padding: "2rem" }}>
      <h1>__TITLE__</h1>
      <p>API health: {status}</p>
    </main>
  );
}
"#;

fn copy_template_dir(src: &Path, dst: &Path) -> NestResult<()> {
    if !src.exists() {
        return Err(NestError::io(format!(
            "Template directory not found: {}",
            src.display()
        )));
    }

    fs::create_dir_all(dst)
        .map_err(|e| NestError::io(format!("failed to create {}: {e}", dst.display())))?;

    for entry in fs::read_dir(src)
        .map_err(|e| NestError::io(format!("failed to read {}: {e}", src.display())))?
    {
        let entry = entry.map_err(|e| NestError::io(format!("failed to read entry: {e}")))?;
        let src_path = entry.path();
        let file_name = entry.file_name();

        // Skip node_modules and other large directories
        if file_name == "node_modules" || file_name == "target" || file_name == "dist" {
            continue;
        }

        let dst_path = dst.join(&file_name);

        if src_path.is_dir() {
            copy_template_dir(&src_path, &dst_path)?;
        } else {
            copy_template_file(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn copy_template_file(src: &Path, dst: &Path) -> NestResult<()> {
    // Skip files that shouldn't be copied
    let file_name = src.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name == ".taurignore" || file_name == "nest-app.toml" {
        return Ok(());
    }

    fs::copy(src, dst)
        .map_err(|e| NestError::io(format!(
            "failed to copy {} to {}: {e}",
            src.display(),
            dst.display()
        )))?;

    Ok(())
}

fn title_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;

    for c in s.chars() {
        if c == '-' || c == '_' {
            result.push(' ');
            capitalize = true;
        } else if capitalize {
            result.extend(c.to_uppercase());
            capitalize = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Create the Tauri plugin for new app commands.
pub fn new_app_plugin<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("new-app")
        .invoke_handler(tauri::generate_handler![
            new_app_list_crates,
            new_app_crate_profile,
            new_app_scaffold,
            new_app_build,
        ])
        .build()
}

use tauri::plugin::TauriPlugin;
