use kiwi_plugin_api::PluginCommand;
use libloading::Library;

/// Keeps plugin dynamic libraries loaded for the process lifetime.
#[derive(Default)]
pub struct PluginHost {
    plugins: Vec<LoadedPlugin>,
}

impl PluginHost {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn push(&mut self, plugin: LoadedPlugin) {
        self.plugins.push(plugin);
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

pub struct LoadedPlugin {
    pub name: String,
    pub version: String,
    pub commands: Vec<PluginCommand>,
    pub library: Library,
}

impl std::fmt::Debug for LoadedPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedPlugin")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("commands", &self.commands)
            .finish_non_exhaustive()
    }
}
