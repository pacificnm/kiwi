/// Commands exposed by the inline `kiwi` Tauri plugin (see `commands.rs`).
///
/// Listing them here lets `tauri-build` autogenerate `allow-*`/`deny-*` ACL
/// permissions and a `kiwi:default` set. Without this, Tauri v2 denies every
/// `plugin:kiwi|*` invoke ("plugin not found"). Keep in sync with
/// `kiwi_plugin`'s `generate_handler!`.
const KIWI_COMMANDS: &[&str] = &[
    "kiwi_host_info",
    "logs_snapshot",
    "logs_clear",
    "problems_snapshot",
    "problems_run",
    "docs_list",
    "docs_read",
    "swift_status",
    "swift_tasks_overview",
    "swift_list_projects",
    "swift_link_workspace",
    "swift_unlink_workspace",
    "swift_get_task",
    "mcp_overview",
    "agent_launch",
    "agent_input",
    "agent_resize",
    "agent_stop",
    "agent_status",
    "agent_settings_get",
    "agent_settings_save",
    "ollama_list_models",
    "ollama_auth_status",
    "ollama_signin",
    "ollama_signout",
    "codex_account_status",
    "codex_login",
    "codex_logout",
    "terminal_open",
    "terminal_input",
    "terminal_resize",
    "terminal_close",
    "terminal_list",
    "workspace_info",
    "workspace_list",
    "workspace_read",
    "workspace_write",
    "workspace_create_file",
    "workspace_create_dir",
    "workspace_rename",
    "workspace_delete",
    "workspace_copy",
    "workspace_reveal",
    "workspace_open",
    "workspace_search",
    "workspace_replace_all",
    "git_status",
    "git_stage",
    "git_stage_all",
    "git_unstage",
    "git_discard",
    "git_commit",
    "git_push",
    "git_pull",
    "git_log",
    "git_commit_changes",
    "github_auth_status",
    "github_repo",
    "github_issue_list",
    "github_issue_view",
    "github_issue_create",
    "github_issue_comment",
    "github_label_list",
    "github_milestone_list",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().plugin(
            "kiwi",
            tauri_build::InlinedPlugin::new()
                .commands(KIWI_COMMANDS)
                .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
        ),
    )
    .expect("failed to run tauri-build");
}
