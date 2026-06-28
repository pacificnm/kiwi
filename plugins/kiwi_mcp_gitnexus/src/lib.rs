use kiwi_plugin_api::{declare_plugin, KiwiPlugin, PluginApi, PluginResult};

#[derive(Default)]
struct GiteaPlugin;

impl KiwiPlugin for GiteaPlugin {
    fn name(&self) -> &'static str {
        "kiwi-mcp-gitnexus"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn register(&self, api: &mut PluginApi<'_>) {
        api.register_command("gitea.list_issues",    "Gitea: List Issues",          gitea_list_issues);
        api.register_command("gitea.create_issue",   "Gitea: Create Issue",         gitea_create_issue);
        api.register_command("gitea.create_pr",      "Gitea: Create Pull Request",  gitea_create_pr);
        api.register_command("gitea.create_branch",  "Gitea: Create Branch",        gitea_create_branch);
        api.register_command("gitea.list_branches",  "Gitea: List Branches",        gitea_list_branches);
        api.register_command("gitea.milestone",      "Gitea: Create Milestone",     gitea_milestone);
    }
}

extern "C" fn gitea_list_issues()  -> PluginResult { PluginResult::Ok }
extern "C" fn gitea_create_issue() -> PluginResult { PluginResult::Ok }
extern "C" fn gitea_create_pr()    -> PluginResult { PluginResult::Ok }
extern "C" fn gitea_create_branch()-> PluginResult { PluginResult::Ok }
extern "C" fn gitea_list_branches()-> PluginResult { PluginResult::Ok }
extern "C" fn gitea_milestone()    -> PluginResult { PluginResult::Ok }

declare_plugin!(GiteaPlugin);
