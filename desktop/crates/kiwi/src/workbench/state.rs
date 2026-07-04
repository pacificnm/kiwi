//! Workbench layout state.

use nest_ai::ChatRole;
use nest_ai::CompletionMetrics;

use super::activity::Activity;
use super::bottom_panel::BottomTab;
use super::editor::EditorState;
use super::explorer::ExplorerState;
use super::prompt::PromptDraft;
use crate::agent::AgentSettings;
use crate::project::{merged_ignore, ProjectConfig};

/// One message in the AI chat panel.
#[derive(Debug, Clone)]
pub struct ChatEntry {
    /// Message author role.
    pub role: ChatRole,
    /// Message body.
    pub content: String,
}

/// One tool invocation in the Tool Activity panel.
#[derive(Debug, Clone)]
pub struct ToolActivityEntry {
    /// Model-visible tool name.
    pub tool: String,
    /// Arguments, result preview, or error text.
    pub detail: String,
    /// Whether the tool is still running.
    pub running: bool,
}

/// Workbench UI state (layout MVP placeholders).
#[derive(Debug, Clone)]
pub struct WorkbenchState {
    /// Selected activity bar item.
    pub activity: Activity,
    /// Selected bottom panel tab.
    pub bottom_tab: BottomTab,
    /// Editor tabs and active file.
    pub editor: EditorState,
    /// Project file tree.
    pub explorer: ExplorerState,
    /// AI prompt draft.
    pub prompt: PromptDraft,
    /// Active LLM model label.
    pub model: String,
    /// Open project configuration.
    pub project: ProjectConfig,
    /// Sidebar search query (Search activity).
    pub search_query: String,
    /// Conversation history for the AI panel.
    pub chat_messages: Vec<ChatEntry>,
    /// Whether a completion request is in flight.
    pub chat_busy: bool,
    /// Last chat error message, if any.
    pub chat_error: Option<String>,
    /// Token and timing stats from the last completed response.
    pub chat_metrics: Option<CompletionMetrics>,
    /// Agent endpoint and model configuration.
    pub agent: AgentSettings,
    /// When true, Send runs the MCP agent loop instead of plain chat.
    pub agent_mode: bool,
    /// Recent MCP tool invocations for the bottom panel.
    pub tool_activity: Vec<ToolActivityEntry>,
    /// Resolved MCP config path for display.
    pub agent_mcp_path: String,
    /// MCP server ids from config.
    pub agent_mcp_servers: Vec<String>,
}

impl Default for WorkbenchState {
    fn default() -> Self {
        Self::demo()
    }
}

impl WorkbenchState {
    /// Demo state for the layout shell.
    pub fn demo() -> Self {
        let project = ProjectConfig {
            root: std::path::PathBuf::from("."),
            name: "kiwi".into(),
            ignore: merged_ignore(None),
        };
        Self {
            activity: Activity::Explorer,
            bottom_tab: BottomTab::Terminal,
            editor: EditorState::empty(),
            explorer: ExplorerState::new(&project.root, &project.name, project.ignore.clone()),
            prompt: PromptDraft::default(),
            model: "smollm2:360m".into(),
            project,
            search_query: String::new(),
            chat_messages: Vec::new(),
            chat_busy: false,
            chat_error: None,
            chat_metrics: None,
            agent: AgentSettings::default(),
            agent_mode: false,
            tool_activity: Vec::new(),
            agent_mcp_path: String::new(),
            agent_mcp_servers: Vec::new(),
        }
    }

    /// Keeps title-bar model label in sync with agent settings.
    pub fn sync_model_from_agent(&mut self) {
        self.model = self.agent.model.clone();
    }
}
