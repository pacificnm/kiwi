use kiwi_plugin_api::{declare_plugin, KiwiPlugin, PluginApi, PluginResult};

#[derive(Default)]
struct GitPlugin;

impl KiwiPlugin for GitPlugin {
    fn name(&self) -> &'static str {
        "kiwi-mcp-git"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn register(&self, api: &mut PluginApi<'_>) {
        api.register_command("git.status", "Git: Show Status", git_status);
        api.register_command("git.commit", "Git: Commit", git_commit);
        api.register_command("git.branch", "Git: Create Branch", git_branch);
        api.register_command("github.issue", "GitHub: Create Issue", github_issue);
        api.register_command("github.pr", "GitHub: Create Pull Request", github_pr);
        api.register_command("github.milestone", "GitHub: Create Milestone", github_milestone);
    }
}

extern "C" fn git_status() -> PluginResult { PluginResult::Ok }
extern "C" fn git_commit() -> PluginResult { PluginResult::Ok }
extern "C" fn git_branch() -> PluginResult { PluginResult::Ok }
extern "C" fn github_issue() -> PluginResult { PluginResult::Ok }
extern "C" fn github_pr() -> PluginResult { PluginResult::Ok }
extern "C" fn github_milestone() -> PluginResult { PluginResult::Ok }

declare_plugin!(GitPlugin);
