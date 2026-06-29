mod discovery;
mod load;
pub mod registry;

pub use load::load_plugins;
pub use registry::{default_registry_path, PluginRegistry};
