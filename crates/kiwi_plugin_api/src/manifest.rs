use serde::Deserialize;

/// Metadata from a plugin's `plugin.toml` manifest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub min_kiwi_version: String,
    /// Dynamic-library entry symbol. Defaults to [`crate::PLUGIN_INIT_SYMBOL`].
    #[serde(default = "default_entry_symbol")]
    pub entry: String,
}

fn default_entry_symbol() -> String {
    crate::PLUGIN_INIT_SYMBOL.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_manifest_with_defaults() {
        let manifest: PluginManifest = toml::from_str(
            r#"
            name = "hello"
            version = "0.1.0"
            min_kiwi_version = "0.1.0"
            "#,
        )
        .expect("manifest");
        assert_eq!(manifest.name, "hello");
        assert_eq!(manifest.entry, "kiwi_plugin_init");
    }
}
