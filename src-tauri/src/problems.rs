//! Workspace diagnostics for the Problems panel (Rust, TypeScript, ESLint).
//!
//! Runs `cargo check`, `cargo clippy`, `tsc --noEmit`, and `eslint` where
//! applicable, parses structured output, and stores normalized diagnostics.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use ignore::WalkBuilder;
use nest_error::{NestError, NestResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MAX_TSCONFIGS: usize = 12;
const MAX_DIAGNOSTICS: usize = 2_000;

const IGNORED_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".venv",
    "dist",
    "build",
    ".tauri",
    ".next",
];

/// One problem row in the Problems panel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProblemDiagnostic {
    pub id: String,
    pub source: String,
    pub severity: String,
    pub message: String,
    pub rel_path: String,
    pub line: u32,
    pub col: u32,
    pub end_line: Option<u32>,
    pub end_col: Option<u32>,
    pub code: Option<String>,
}

/// Latest diagnostics snapshot for the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemsReport {
    pub diagnostics: Vec<ProblemDiagnostic>,
    pub error_count: u32,
    pub warning_count: u32,
    pub info_count: u32,
    pub running: bool,
    pub ran_at: Option<String>,
    pub summary: String,
}

impl Default for ProblemsReport {
    fn default() -> Self {
        Self {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            running: false,
            ran_at: None,
            summary: "No diagnostics run yet.".into(),
        }
    }
}

/// Managed state for workspace diagnostics.
pub struct ProblemsState {
    inner: Mutex<ProblemsReport>,
}

impl Default for ProblemsState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(ProblemsReport::default()),
        }
    }
}

impl ProblemsState {
    pub fn snapshot(&self) -> ProblemsReport {
        self.inner.lock().expect("problems mutex").clone()
    }

    pub fn set_running(&self, running: bool) {
        let mut report = self.inner.lock().expect("problems mutex");
        report.running = running;
        if running {
            report.summary = "Running diagnostics…".into();
        }
    }

    pub fn set_report(&self, diagnostics: Vec<ProblemDiagnostic>, summary: String) {
        let mut report = self.inner.lock().expect("problems mutex");
        report.error_count = diagnostics
            .iter()
            .filter(|item| item.severity == "error")
            .count() as u32;
        report.warning_count = diagnostics
            .iter()
            .filter(|item| item.severity == "warning")
            .count() as u32;
        report.info_count = diagnostics
            .iter()
            .filter(|item| item.severity == "info")
            .count() as u32;
        report.diagnostics = diagnostics;
        report.running = false;
        report.ran_at = Some(now_iso());
        report.summary = summary;
    }
}

/// Collects diagnostics for the open workspace.
pub fn collect(workspace_root: &Path) -> NestResult<Vec<ProblemDiagnostic>> {
    let mut diagnostics = Vec::new();

    if let Some(cargo_root) = find_cargo_root(workspace_root) {
        diagnostics.extend(run_cargo_check(&cargo_root, workspace_root)?);
        diagnostics.extend(run_cargo_clippy(&cargo_root, workspace_root)?);
    }

    for tsconfig in discover_tsconfigs(workspace_root) {
        diagnostics.extend(run_tsc(&tsconfig, workspace_root)?);
    }

    for eslint_root in discover_eslint_roots(workspace_root) {
        diagnostics.extend(run_eslint(&eslint_root, workspace_root)?);
    }

    diagnostics.sort_by(|a, b| {
        a.rel_path
            .cmp(&b.rel_path)
            .then(a.line.cmp(&b.line))
            .then(a.col.cmp(&b.col))
            .then(a.source.cmp(&b.source))
    });

    if diagnostics.len() > MAX_DIAGNOSTICS {
        diagnostics.truncate(MAX_DIAGNOSTICS);
    }

    Ok(diagnostics)
}

fn find_cargo_root(workspace_root: &Path) -> Option<PathBuf> {
    let manifest = workspace_root.join("Cargo.toml");
    if manifest.is_file() {
        return Some(workspace_root.to_path_buf());
    }
    None
}

fn is_workspace_manifest(path: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    text.contains("[workspace]")
}

fn discover_tsconfigs(workspace_root: &Path) -> Vec<PathBuf> {
    let mut configs = Vec::new();
    for entry in WalkBuilder::new(workspace_root)
        .hidden(false)
        .filter_entry(|entry| {
            if !entry.file_type().is_some_and(|kind| kind.is_dir()) {
                return true;
            }
            !IGNORED_DIRS.contains(&entry.file_name().to_string_lossy().as_ref())
        })
        .build()
        .flatten()
    {
        if entry.file_type().is_some_and(|kind| kind.is_file())
            && entry.file_name() == "tsconfig.json"
        {
            configs.push(entry.into_path());
            if configs.len() >= MAX_TSCONFIGS {
                break;
            }
        }
    }
    configs.sort();
    configs
}

fn discover_eslint_roots(workspace_root: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for entry in WalkBuilder::new(workspace_root)
        .hidden(false)
        .max_depth(Some(6))
        .filter_entry(|entry| {
            if !entry.file_type().is_some_and(|kind| kind.is_dir()) {
                return true;
            }
            !IGNORED_DIRS.contains(&entry.file_name().to_string_lossy().as_ref())
        })
        .build()
        .flatten()
    {
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        let is_config = name == "eslint.config.js"
            || name == "eslint.config.mjs"
            || name == "eslint.config.cjs"
            || name.starts_with(".eslintrc");
        if !is_config {
            continue;
        }
        let dir = entry.path().parent().unwrap_or(workspace_root).to_path_buf();
        if !roots.contains(&dir) {
            roots.push(dir);
        }
    }
    roots.sort();
    roots
}

fn run_cargo_check(cargo_root: &Path, workspace_root: &Path) -> NestResult<Vec<ProblemDiagnostic>> {
    let mut command = Command::new("cargo");
    command
        .arg("check")
        .arg("--message-format=json")
        .current_dir(cargo_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if is_workspace_manifest(&cargo_root.join("Cargo.toml")) {
        command.arg("--workspace");
    }
    run_cargo_json_command(command, "rustc", workspace_root)
}

fn run_cargo_clippy(cargo_root: &Path, workspace_root: &Path) -> NestResult<Vec<ProblemDiagnostic>> {
    let mut command = Command::new("cargo");
    command
        .arg("clippy")
        .arg("--message-format=json")
        .arg("-q")
        .current_dir(cargo_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if is_workspace_manifest(&cargo_root.join("Cargo.toml")) {
        command.arg("--workspace");
    }
    run_cargo_json_command(command, "clippy", workspace_root)
}

fn run_cargo_json_command(
    mut command: Command,
    source: &str,
    workspace_root: &Path,
) -> NestResult<Vec<ProblemDiagnostic>> {
    let output = command
        .output()
        .map_err(|error| NestError::io(format!("failed to run cargo {source}: {error}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut diagnostics = parse_cargo_json_lines(&stdout, source, workspace_root);

    if diagnostics.is_empty() && !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !stderr.is_empty() {
            diagnostics.push(ProblemDiagnostic {
                id: format!("{source}:process"),
                source: source.to_string(),
                severity: "error".into(),
                message: stderr,
                rel_path: ".".into(),
                line: 1,
                col: 1,
                end_line: None,
                end_col: None,
                code: None,
            });
        }
    }

    Ok(diagnostics)
}

fn parse_cargo_json_lines(text: &str, source: &str, workspace_root: &Path) -> Vec<ProblemDiagnostic> {
    let mut diagnostics = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value.get("reason").and_then(Value::as_str) != Some("compiler-message") {
            continue;
        }
        let Some(message) = value.get("message") else {
            continue;
        };
        let level = message
            .get("level")
            .and_then(Value::as_str)
            .unwrap_or("error");
        let severity = match level {
            "warning" => "warning",
            "note" | "help" => "info",
            _ => "error",
        };
        let text = message
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("compiler error")
            .to_string();
        let code = message
            .get("code")
            .and_then(|value| value.get("code"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let spans = message
            .get("spans")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let primary = spans
            .iter()
            .find(|span| span.get("is_primary").and_then(Value::as_bool).unwrap_or(false))
            .or_else(|| spans.first());
        let Some(span) = primary else {
            continue;
        };
        let file_name = span
            .get("file_name")
            .and_then(Value::as_str)
            .unwrap_or("");
        if file_name.is_empty() {
            continue;
        }
        let rel_path = rel_path_from_abs(file_name, workspace_root);
        let line = span
            .get("line_start")
            .and_then(Value::as_u64)
            .unwrap_or(1) as u32;
        let col = span
            .get("column_start")
            .and_then(Value::as_u64)
            .unwrap_or(1) as u32;
        let end_line = span.get("line_end").and_then(Value::as_u64).map(|v| v as u32);
        let end_col = span
            .get("column_end")
            .and_then(Value::as_u64)
            .map(|v| v as u32);
        diagnostics.push(ProblemDiagnostic {
            id: format!("{source}:{rel_path}:{line}:{col}:{}", diagnostics.len()),
            source: source.to_string(),
            severity: severity.to_string(),
            message: text,
            rel_path,
            line,
            col,
            end_line,
            end_col,
            code,
        });
    }
    diagnostics
}

fn run_tsc(tsconfig: &Path, workspace_root: &Path) -> NestResult<Vec<ProblemDiagnostic>> {
    let project_dir = tsconfig
        .parent()
        .ok_or_else(|| NestError::validation("invalid tsconfig path"))?;
    let output = Command::new("npx")
        .args([
            "--yes",
            "tsc",
            "--noEmit",
            "--pretty",
            "false",
            "-p",
            tsconfig.to_str().unwrap_or("tsconfig.json"),
        ])
        .current_dir(project_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| NestError::io(format!("failed to run tsc: {error}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    Ok(parse_tsc_output(
        &combined,
        workspace_root,
        &rel_path_from_abs(
            project_dir.to_string_lossy().as_ref(),
            workspace_root,
        ),
    ))
}

fn parse_tsc_output(text: &str, workspace_root: &Path, project_rel: &str) -> Vec<ProblemDiagnostic> {
    let re = Regex::new(
        r"^(?P<file>[^\s(]+)\((?P<line>\d+),(?P<col>\d+)\): (?P<level>error|warning) TS(?P<code>\d+): (?P<message>.+)$",
    )
    .expect("tsc regex");
    let mut diagnostics = Vec::new();
    for line in text.lines() {
        let Some(caps) = re.captures(line.trim()) else {
            continue;
        };
        let file = caps.name("file").map(|m| m.as_str()).unwrap_or("");
        let rel_path = if Path::new(file).is_absolute() {
            rel_path_from_abs(file, workspace_root)
        } else if project_rel == "." {
            normalize_rel_path(file)
        } else {
            format!("{project_rel}/{}", normalize_rel_path(file))
        };
        let line = caps.name("line").map(|m| m.as_str()).unwrap_or("1");
        let col = caps.name("col").map(|m| m.as_str()).unwrap_or("1");
        diagnostics.push(ProblemDiagnostic {
            id: format!("tsc:{rel_path}:{line}:{col}"),
            source: "tsc".into(),
            severity: caps["level"].to_string(),
            message: caps["message"].to_string(),
            rel_path,
            line: line.parse().unwrap_or(1),
            col: col.parse().unwrap_or(1),
            end_line: None,
            end_col: None,
            code: Some(format!("TS{}", &caps["code"])),
        });
    }
    diagnostics
}

fn run_eslint(eslint_root: &Path, workspace_root: &Path) -> NestResult<Vec<ProblemDiagnostic>> {
    let output = Command::new("npx")
        .args(["--yes", "eslint", ".", "-f", "json", "--no-error-on-unmatched-pattern"])
        .current_dir(eslint_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| NestError::io(format!("failed to run eslint: {error}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_eslint_json(
        &stdout,
        workspace_root,
        &rel_path_from_abs(
            eslint_root.to_string_lossy().as_ref(),
            workspace_root,
        ),
    ))
}

fn parse_eslint_json(text: &str, workspace_root: &Path, _project_rel: &str) -> Vec<ProblemDiagnostic> {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    let Some(files) = value.as_array() else {
        return Vec::new();
    };
    let mut diagnostics = Vec::new();
    for file in files {
        let file_path = file.get("filePath").and_then(Value::as_str).unwrap_or("");
        if file_path.is_empty() {
            continue;
        }
        let rel_path = rel_path_from_abs(file_path, workspace_root);
        let Some(messages) = file.get("messages").and_then(Value::as_array) else {
            continue;
        };
        for message in messages {
            let severity_num = message.get("severity").and_then(Value::as_u64).unwrap_or(2);
            let severity = match severity_num {
                1 => "warning",
                0 => "info",
                _ => "error",
            };
            let line = message.get("line").and_then(Value::as_u64).unwrap_or(1) as u32;
            let col = message.get("column").and_then(Value::as_u64).unwrap_or(1) as u32;
            let end_line = message.get("endLine").and_then(Value::as_u64).map(|v| v as u32);
            let end_col = message.get("endColumn").and_then(Value::as_u64).map(|v| v as u32);
            let text = message
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("lint problem")
                .to_string();
            let code = message
                .get("ruleId")
                .and_then(Value::as_str)
                .map(str::to_string);
            diagnostics.push(ProblemDiagnostic {
                id: format!("eslint:{rel_path}:{line}:{col}:{}", diagnostics.len()),
                source: "eslint".into(),
                severity: severity.to_string(),
                message: text,
                rel_path: rel_path.clone(),
                line,
                col,
                end_line,
                end_col,
                code,
            });
        }
    }
    diagnostics
}

fn rel_path_from_abs(abs: &str, workspace_root: &Path) -> String {
    let path = Path::new(abs);
    if let Ok(stripped) = path.strip_prefix(workspace_root) {
        return normalize_rel_path(&stripped.to_string_lossy());
    }
    normalize_rel_path(abs)
}

fn normalize_rel_path(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cargo_json_diagnostics() {
        let json = r#"{"reason":"compiler-message","message":{"level":"error","message":"mismatch types","code":{"code":"E0308","explanation":null},"spans":[{"file_name":"/tmp/ws/src/main.rs","byte_start":0,"byte_end":1,"line_start":4,"line_end":4,"column_start":9,"column_end":10,"is_primary":true,"text":[{"text":"let x = 1;","highlight_start":9,"highlight_end":10}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}]}}"#;
        let items = parse_cargo_json_lines(json, "rustc", Path::new("/tmp/ws"));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].rel_path, "src/main.rs");
        assert_eq!(items[0].code.as_deref(), Some("E0308"));
        assert_eq!(items[0].severity, "error");
    }

    #[test]
    fn parses_tsc_output() {
        let text = "src/App.tsx(12,5): error TS2304: Cannot find name 'foo'.\n";
        let items = parse_tsc_output(text, Path::new("/tmp/ws"), ".");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].rel_path, "src/App.tsx");
        assert_eq!(items[0].code.as_deref(), Some("TS2304"));
    }

    #[test]
    fn parses_eslint_json() {
        let json = r#"[{"filePath":"/tmp/ws/src/App.tsx","messages":[{"ruleId":"no-unused-vars","severity":2,"message":"'x' is defined but never used.","line":3,"column":7,"endLine":3,"endColumn":8}]}]"#;
        let items = parse_eslint_json(json, Path::new("/tmp/ws"), ".");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].source, "eslint");
        assert_eq!(items[0].severity, "error");
    }
}
