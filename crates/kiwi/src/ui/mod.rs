mod agent;
mod branches;
mod context_menu;
mod diff;
mod file_tree;
mod git;
mod github;
mod label_picker;
mod logs;
mod mouse;
mod mouse_clicks;
mod mouse_scroll;
mod notifications;
mod palette;
mod preview;
mod render;
mod scrollback;
mod scrollbar;
mod search;
mod settings;
mod shell;
mod status_bar;
mod tabs;

pub use agent::{agent_scrollback_area, map_agent_session_click};
pub use branches::{branch_interaction_at, branches_viewport_rows};
pub use context_menu::github_context_menu_item_at;
pub use file_tree::{file_tree_interaction_at, FileTreeMouseAction};
pub use git::{git_interaction_at, git_viewport_rows};
pub use github::{
    github_issue_interaction_at, github_pr_interaction_at, issue_detail_viewport_rows,
    issues_viewport_rows, pr_detail_viewport_rows, prs_viewport_rows,
};
pub use mouse::map_mouse_click;
pub use mouse::mouse_interactions_enabled;
pub use mouse_clicks::{DoubleClickTarget, DoubleClickTracker};
pub use mouse_scroll::{map_mouse_wheel, WheelDirection};
pub use palette::palette_match_at;
pub use render::draw_frame;
pub use search::search_interaction_at;
pub use settings::{settings_interaction_at, settings_viewport_rows};
