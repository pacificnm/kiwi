use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(default)]
pub struct RawConfig {
    pub app: Option<AppSection>,
    pub theme: Option<ThemeSection>,
    pub editor: Option<EditorSection>,
    pub agent: Option<AgentSection>,
    pub shell: Option<ShellSection>,
    pub mouse: Option<MouseSection>,
    pub git: Option<GitSection>,
    pub github: Option<GitHubSection>,
    pub workspace: Option<WorkspaceSection>,
    pub status_bar: Option<StatusBarSection>,
    pub search: Option<SearchSection>,
    pub preview: Option<PreviewSection>,
    pub diff: Option<DiffSection>,
    pub watcher: Option<WatcherSection>,
    pub plugins: Option<PluginsSection>,
    pub gui: Option<GuiSection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct AppSection {
    pub left_width: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct ThemeSection {
    pub name: Option<String>,
    pub custom: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct EditorSection {
    pub command: Option<String>,
    /// When set, overrides automatic terminal vs GUI detection.
    pub terminal: Option<bool>,
}

/// How the agent panel connects to an AI backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    /// Spawn a subprocess and render its PTY output (legacy default).
    #[default]
    Pty,
    /// Call the LLM API directly; render a native chat panel.
    Api,
}

/// Per-provider settings stored under `[agent.providers.<name>]`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct ProviderSection {
    pub api_key_env: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub api_url: Option<String>,
    pub tool_profile: Option<String>,
    /// Ollama: model for tool orchestration (e.g. `llama3.1:8b`).
    pub tool_model: Option<String>,
    /// Ollama: model for code synthesis after tools (e.g. `qwen2.5-coder:7b`).
    pub code_model: Option<String>,
    /// Ollama: embedding model for RAG plugins (e.g. `nomic-embed-text`).
    pub embedding_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct AgentSection {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    /// `"api"` — native LLM API; `"pty"` — subprocess (default).
    pub mode: Option<AgentMode>,
    /// Active provider name (new-style). Mirrors legacy `provider` field.
    pub active: Option<String>,
    /// Per-provider settings keyed by provider name.
    pub providers: Option<HashMap<String, ProviderSection>>,
    // ---- Legacy flat fields (backward compat) ----
    pub provider: Option<String>,
    pub api_key_env: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub api_url: Option<String>,
    pub tool_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct ShellSection {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MouseMode {
    #[default]
    Hybrid,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct MouseSection {
    pub enabled: Option<bool>,
    pub mode: Option<MouseMode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct GitSection {
    pub watch: Option<bool>,
    pub show_untracked: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct GitHubSection {
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct WorkspaceSection {
    pub persist: Option<bool>,
    pub save_interval_secs: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct StatusBarSection {
    pub show_issue: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct SearchSection {
    pub command: Option<String>,
    pub debounce_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct PreviewSection {
    pub max_size_bytes: Option<u64>,
    pub line_numbers: Option<bool>,
    pub wrap: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct WatcherSection {
    pub debounce_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct PluginsSection {
    pub enabled: Option<bool>,
    pub directory: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct DiffSection {
    pub context_lines: Option<u32>,
    pub word_wrap: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(default)]
pub struct GuiSection {
    pub font: Option<GuiFontSection>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(default)]
pub struct GuiFontSection {
    pub size: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub app: AppSettings,
    pub theme: ThemeSettings,
    pub editor: EditorSettings,
    pub agent: AgentSettings,
    pub shell: ShellSettings,
    pub mouse: MouseSettings,
    pub git: GitSettings,
    pub github: GitHubSettings,
    pub workspace: WorkspaceSettings,
    pub status_bar: StatusBarSettings,
    pub search: SearchSettings,
    pub preview: PreviewSettings,
    pub diff: DiffSettings,
    pub watcher: WatcherSettings,
    pub plugins: PluginsSettings,
    pub gui: GuiSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSettings {
    pub left_width: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeSettings {
    pub name: String,
    pub custom: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditorSettings {
    /// Explicit `[editor] command` from config; when unset, runtime resolution uses
    /// `$VISUAL` → `$EDITOR` → `nano` (ADR-013).
    pub configured_command: Option<String>,
    /// When set, forces terminal suspend/resume or detached GUI launch.
    pub terminal: Option<bool>,
}

/// Resolved per-provider settings. Looked up by provider name at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSettings {
    /// Env var that holds the API key (e.g. `"ANTHROPIC_API_KEY"`).
    pub api_key_env: String,
    /// Literal API key from config file (fallback; prefer env var).
    pub api_key: Option<String>,
    /// Model to request from this provider.
    pub model: String,
    /// Base URL for the provider's API endpoint.
    pub api_url: Option<String>,
    /// Optional per-provider tool profile override.
    pub tool_profile: Option<String>,
    /// Ollama: model for tool orchestration (e.g. `llama3.1:8b`).
    pub tool_model: Option<String>,
    /// Ollama: model for code synthesis after tools (e.g. `qwen2.5-coder:7b`).
    pub code_model: Option<String>,
    /// Ollama: embedding model for RAG plugins (e.g. `nomic-embed-text`).
    pub embedding_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSettings {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub mode: AgentMode,
    /// Which provider is currently active (e.g. `"claude"`, `"openai"`, `"ollama"`).
    pub active_provider: Option<String>,
    /// Per-provider settings — each entry survives switching to another provider.
    pub providers: HashMap<String, ProviderSettings>,
    /// Default tool profile for API agents (`coding`, `all`, etc.).
    pub tool_profile: String,
    // ---- Legacy flat fields kept for backward compat ----
    pub api_key_env: String,
    pub api_key: Option<String>,
    pub model: String,
    pub api_url: Option<String>,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            command: "agent".to_string(),
            args: Vec::new(),
            env: HashMap::new(),
            mode: AgentMode::Pty,
            active_provider: None,
            providers: HashMap::new(),
            tool_profile: "all".to_string(),
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
            api_key: None,
            model: "claude-opus-4-8".to_string(),
            api_url: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellSettings {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseSettings {
    pub enabled: bool,
    pub mode: MouseMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitSettings {
    pub watch: bool,
    pub show_untracked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubSettings {
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSettings {
    pub persist: bool,
    pub save_interval_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarSettings {
    pub show_issue: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchSettings {
    pub command: String,
    pub debounce_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSettings {
    pub max_size_bytes: u64,
    pub line_numbers: bool,
    pub wrap: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffSettings {
    pub context_lines: u32,
    pub word_wrap: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatcherSettings {
    pub debounce_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginsSettings {
    pub enabled: bool,
    pub directory: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GuiSettings {
    pub font_size: f32,
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self { font_size: 14.0 }
    }
}

impl Eq for GuiSettings {}

impl PartialEq for GuiSettings {
    fn eq(&self, other: &Self) -> bool {
        self.font_size.to_bits() == other.font_size.to_bits()
    }
}

impl Default for ResolvedConfig {
    fn default() -> Self {
        Self {
            app: AppSettings { left_width: 30 },
            theme: ThemeSettings {
                name: "kiwi-dark".to_string(),
                custom: None,
            },
            editor: EditorSettings::default(),
            agent: AgentSettings::default(),
            shell: ShellSettings {
                command: default_shell_command(),
                args: Vec::new(),
            },
            mouse: MouseSettings {
                enabled: true,
                mode: MouseMode::Hybrid,
            },
            git: GitSettings {
                watch: true,
                show_untracked: true,
            },
            github: GitHubSettings {
                command: "gh".to_string(),
            },
            workspace: WorkspaceSettings {
                persist: true,
                save_interval_secs: 30,
            },
            status_bar: StatusBarSettings { show_issue: true },
            search: SearchSettings {
                command: "rg".to_string(),
                debounce_ms: 200,
            },
            preview: PreviewSettings {
                max_size_bytes: 1_048_576,
                line_numbers: true,
                wrap: false,
            },
            diff: DiffSettings {
                context_lines: 3,
                word_wrap: false,
            },
            watcher: WatcherSettings { debounce_ms: 300 },
            plugins: PluginsSettings {
                enabled: true,
                directory: default_plugins_directory(None),
            },
            gui: GuiSettings::default(),
        }
    }
}

pub fn default_plugins_directory(home: Option<&Path>) -> PathBuf {
    home.map(|dir| dir.join(".config/kiwi/plugins"))
        .unwrap_or_else(|| PathBuf::from(".config/kiwi/plugins"))
}

/// Default directory for plugin-provided executables (`~/.local/bin`).
pub fn default_plugin_bin_directory(home: Option<&Path>) -> PathBuf {
    home.map(|dir| dir.join(".local/bin"))
        .unwrap_or_else(|| PathBuf::from(".local/bin"))
}

fn default_shell_command() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string())
}

impl RawConfig {
    pub fn apply_to(&self, resolved: &mut ResolvedConfig, home: Option<&Path>) {
        if let Some(app) = &self.app {
            if let Some(left_width) = app.left_width {
                resolved.app.left_width = left_width;
            }
        }

        if let Some(theme) = &self.theme {
            if let Some(name) = &theme.name {
                resolved.theme.name = name.clone();
            }
            if let Some(custom) = &theme.custom {
                resolved.theme.custom = Some(expand_tilde(custom, home));
            }
        }

        if let Some(editor) = &self.editor {
            if let Some(command) = &editor.command {
                resolved.editor.configured_command = Some(command.clone());
            }
            if let Some(terminal) = editor.terminal {
                resolved.editor.terminal = Some(terminal);
            }
        }

        if let Some(agent) = &self.agent {
            if let Some(command) = &agent.command {
                resolved.agent.command = command.clone();
            }
            if let Some(args) = &agent.args {
                resolved.agent.args = args.clone();
            }
            if let Some(env) = &agent.env {
                resolved.agent.env = env.clone();
            }
            if let Some(mode) = agent.mode {
                resolved.agent.mode = mode;
            }
            // New-style: [agent] active = "claude"
            if let Some(active) = &agent.active {
                resolved.agent.active_provider = Some(active.clone());
            }
            // New-style: [agent.providers.<name>] sections
            if let Some(providers) = &agent.providers {
                for (name, section) in providers {
                    let entry = resolved
                        .agent
                        .providers
                        .entry(name.clone())
                        .or_insert_with(|| ProviderSettings {
                            api_key_env: "ANTHROPIC_API_KEY".to_string(),
                            api_key: None,
                            model: "claude-opus-4-8".to_string(),
                            api_url: None,
                            tool_profile: None,
                            tool_model: None,
                            code_model: None,
                            embedding_model: None,
                        });
                    if let Some(v) = &section.api_key_env {
                        entry.api_key_env = v.clone();
                    }
                    if let Some(v) = &section.api_key {
                        entry.api_key = Some(v.clone());
                    }
                    if let Some(v) = &section.model {
                        entry.model = v.clone();
                    }
                    if let Some(v) = &section.api_url {
                        entry.api_url = Some(v.clone());
                    }
                    if let Some(v) = &section.tool_profile {
                        entry.tool_profile = Some(v.clone());
                    }
                    if let Some(v) = &section.embedding_model {
                        entry.embedding_model = Some(v.clone());
                    }
                }
                // Ollama uses a single chat model; ignore legacy split-model fields.
                if let Some(ollama) = resolved.agent.providers.get_mut("ollama") {
                    ollama.tool_model = None;
                    ollama.code_model = None;
                }
            }
            // Legacy flat fields — only migrate non-key fields (api_key_env, model, api_url)
            // into the active provider's entry. The api_key is intentionally NOT migrated
            // here because the flat [agent].api_key is provider-agnostic and putting it into
            // providers[active] would corrupt other providers' entries when switching.
            let provider_name = agent.active.as_ref().or(agent.provider.as_ref());
            if let Some(name) = provider_name {
                let entry = resolved
                    .agent
                    .providers
                    .entry(name.clone())
                    .or_insert_with(|| ProviderSettings {
                        api_key_env: "ANTHROPIC_API_KEY".to_string(),
                        api_key: None,
                        model: "claude-opus-4-8".to_string(),
                        api_url: None,
                        tool_profile: None,
                        tool_model: None,
                        code_model: None,
                        embedding_model: None,
                    });
                if let Some(v) = &agent.api_key_env {
                    entry.api_key_env = v.clone();
                }
                if let Some(v) = &agent.model {
                    entry.model = v.clone();
                }
                if let Some(v) = &agent.api_url {
                    entry.api_url = Some(v.clone());
                }
                // Keep active_provider in sync when only legacy field is present
                if resolved.agent.active_provider.is_none() && agent.provider.is_some() {
                    resolved.agent.active_provider = Some(name.clone());
                }
            }
            // Legacy flat fields also kept for services.rs fallback
            if let Some(api_key_env) = &agent.api_key_env {
                resolved.agent.api_key_env = api_key_env.clone();
            }
            if let Some(api_key) = &agent.api_key {
                resolved.agent.api_key = Some(api_key.clone());
            }
            if let Some(model) = &agent.model {
                resolved.agent.model = model.clone();
            }
            if let Some(api_url) = &agent.api_url {
                resolved.agent.api_url = Some(api_url.clone());
            }
            if let Some(tool_profile) = &agent.tool_profile {
                resolved.agent.tool_profile = tool_profile.clone();
            }
        }

        if let Some(shell) = &self.shell {
            if let Some(command) = &shell.command {
                resolved.shell.command = command.clone();
            }
            if let Some(args) = &shell.args {
                resolved.shell.args = args.clone();
            }
        }

        if let Some(mouse) = &self.mouse {
            if let Some(enabled) = mouse.enabled {
                resolved.mouse.enabled = enabled;
            }
            if let Some(mode) = mouse.mode {
                resolved.mouse.mode = mode;
            }
        }

        if let Some(git) = &self.git {
            if let Some(watch) = git.watch {
                resolved.git.watch = watch;
            }
            if let Some(show_untracked) = git.show_untracked {
                resolved.git.show_untracked = show_untracked;
            }
        }

        if let Some(github) = &self.github {
            if let Some(command) = &github.command {
                resolved.github.command = command.clone();
            }
        }

        if let Some(workspace) = &self.workspace {
            if let Some(persist) = workspace.persist {
                resolved.workspace.persist = persist;
            }
            if let Some(save_interval_secs) = workspace.save_interval_secs {
                resolved.workspace.save_interval_secs = save_interval_secs;
            }
        }

        if let Some(status_bar) = &self.status_bar {
            if let Some(show_issue) = status_bar.show_issue {
                resolved.status_bar.show_issue = show_issue;
            }
        }

        if let Some(search) = &self.search {
            if let Some(command) = &search.command {
                resolved.search.command = command.clone();
            }
            if let Some(debounce_ms) = search.debounce_ms {
                resolved.search.debounce_ms = debounce_ms;
            }
        }

        if let Some(preview) = &self.preview {
            if let Some(max_size_bytes) = preview.max_size_bytes {
                resolved.preview.max_size_bytes = max_size_bytes;
            }
            if let Some(line_numbers) = preview.line_numbers {
                resolved.preview.line_numbers = line_numbers;
            }
            if let Some(wrap) = preview.wrap {
                resolved.preview.wrap = wrap;
            }
        }

        if let Some(diff) = &self.diff {
            if let Some(context_lines) = diff.context_lines {
                resolved.diff.context_lines = context_lines;
            }
            if let Some(word_wrap) = diff.word_wrap {
                resolved.diff.word_wrap = word_wrap;
            }
        }

        if let Some(watcher) = &self.watcher {
            if let Some(debounce_ms) = watcher.debounce_ms {
                resolved.watcher.debounce_ms = debounce_ms;
            }
        }

        if let Some(plugins) = &self.plugins {
            if let Some(enabled) = plugins.enabled {
                resolved.plugins.enabled = enabled;
            }
            if let Some(directory) = &plugins.directory {
                resolved.plugins.directory = expand_tilde(directory, home);
            }
        }

        if let Some(gui) = &self.gui {
            if let Some(font) = &gui.font {
                if let Some(size) = font.size {
                    resolved.gui.font_size = size;
                }
            }
        }
    }
}

pub fn expand_tilde(path: &str, home: Option<&Path>) -> PathBuf {
    let home = home.map(Path::to_path_buf).or_else(home_dir);

    if let Some(suffix) = path.strip_prefix("~/") {
        home.map(|dir| dir.join(suffix))
            .unwrap_or_else(|| PathBuf::from(path))
    } else if path == "~" {
        home.unwrap_or_else(|| PathBuf::from(path))
    } else {
        PathBuf::from(path)
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
