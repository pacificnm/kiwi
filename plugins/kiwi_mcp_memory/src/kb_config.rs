use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct KbConfig {
    pub collections: Vec<CollectionConfig>,
}

#[derive(Deserialize)]
pub struct CollectionConfig {
    pub name: String,
    pub source: String,
    /// File extensions to index (e.g. ["md", "html"]). None = all readable UTF-8 files.
    pub extensions: Option<Vec<String>>,
}

impl KbConfig {
    pub fn from_file(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read kb config: {}", path.display()))?;
        toml::from_str(&text)
            .with_context(|| format!("failed to parse kb config: {}", path.display()))
    }
}
