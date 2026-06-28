use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::ollama::OllamaClient;

const CHUNK_SIZE: usize = 1800;
const CHUNK_OVERLAP: usize = 200;
const MAX_FILES: usize = 2000;

const INDEXED_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "c", "cpp", "h", "hpp", "md", "toml",
    "yaml", "yml", "json",
];

const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".venv",
    "__pycache__",
    "dist",
    "build",
    ".cache",
    ".mypy_cache",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexEntry {
    path: String,
    chunk_index: usize,
    content: String,
    embedding: Vec<f32>,
    mtime_secs: i64,
}

pub struct RagIndex {
    entries: Vec<IndexEntry>,
    client: OllamaClient,
}

impl RagIndex {
    /// Load or build the index. Calls embed API for stale/new files.
    /// Returns Err if the embed model is unavailable.
    pub fn build_or_load(
        repo_root: PathBuf,
        cache_path: PathBuf,
        client: OllamaClient,
    ) -> Result<Self> {
        // Verify embedding works before starting
        client.embed("ping").map_err(|e| {
            anyhow::anyhow!(
                "RAG disabled: embedding failed (pull nomic-embed-text?): {e}"
            )
        })?;

        let mut entries: Vec<IndexEntry> = load_cache(&cache_path).unwrap_or_default();

        let eligible = collect_eligible_files(&repo_root);
        if eligible.len() >= MAX_FILES {
            eprintln!(
                "warning: repository has {} eligible files; indexing first {}",
                eligible.len(),
                MAX_FILES
            );
        }

        let eligible: Vec<_> = eligible.into_iter().take(MAX_FILES).collect();

        // Remove stale entries for files that no longer exist or have changed mtime
        entries.retain(|e| {
            eligible
                .iter()
                .any(|(p, mtime)| p.to_string_lossy() == e.path && *mtime == e.mtime_secs)
        });

        let indexed_paths: std::collections::HashSet<String> =
            entries.iter().map(|e| e.path.clone()).collect();

        let mut added = 0usize;
        for (abs_path, mtime) in &eligible {
            let rel = abs_path
                .strip_prefix(&repo_root)
                .unwrap_or(abs_path)
                .to_string_lossy()
                .into_owned();

            if indexed_paths.contains(&rel) {
                continue;
            }

            let content = match fs::read_to_string(abs_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let chunks = chunk_text(&content, CHUNK_SIZE, CHUNK_OVERLAP);
            for (chunk_index, chunk) in chunks.into_iter().enumerate() {
                match client.embed(&chunk) {
                    Ok(embedding) => {
                        entries.push(IndexEntry {
                            path: rel.clone(),
                            chunk_index,
                            content: chunk,
                            embedding,
                            mtime_secs: *mtime,
                        });
                        added += 1;
                    }
                    Err(e) => {
                        eprintln!("warning: skipping {rel} chunk {chunk_index}: {e}");
                    }
                }
            }
        }

        if added > 0 {
            if let Err(e) = save_cache(&cache_path, &entries) {
                eprintln!("warning: could not save RAG cache: {e}");
            }
        }

        Ok(Self { entries, client })
    }

    /// Embed the query and return the top-N matching chunks as formatted strings.
    pub fn retrieve(&self, query: &str, top_n: usize) -> Vec<String> {
        let embedding = match self.client.embed(query) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        let mut scored: Vec<(f32, &IndexEntry)> = self
            .entries
            .iter()
            .map(|e| (cosine(&embedding, &e.embedding), e))
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(top_n)
            .map(|(_, e)| {
                format!(
                    "--- {} (chunk {}) ---\n{}",
                    e.path, e.chunk_index, e.content
                )
            })
            .collect()
    }

}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let len = text.len();
    if len == 0 {
        return chunks;
    }
    let mut start = 0usize;
    while start < len {
        let raw_end = (start + chunk_size).min(len);
        // Walk back to a valid UTF-8 char boundary
        let end = (raw_end..=len)
            .find(|&i| text.is_char_boundary(i))
            .unwrap_or(len);
        chunks.push(text[start..end].to_string());
        if end >= len {
            break;
        }
        let raw_next = end.saturating_sub(overlap);
        // Walk forward to a valid char boundary
        start = (raw_next..=end)
            .find(|&i| text.is_char_boundary(i))
            .unwrap_or(end);
    }
    chunks
}

fn collect_eligible_files(root: &Path) -> Vec<(PathBuf, i64)> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            e.file_type().is_file()
                || !SKIP_DIRS.iter().any(|skip| {
                    e.file_name()
                        .to_str()
                        .map(|n| n == *skip)
                        .unwrap_or(false)
                })
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| INDEXED_EXTENSIONS.contains(&ext))
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let mtime = e
                .metadata()
                .ok()?
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs() as i64;
            Some((e.into_path(), mtime))
        })
        .collect()
}

fn load_cache(path: &Path) -> Option<Vec<IndexEntry>> {
    let data = fs::read(path).ok()?;
    serde_json::from_slice(&data).ok()
}

fn save_cache(path: &Path, entries: &[IndexEntry]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_vec(entries)?;
    fs::write(&tmp, &data)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_identical_vectors() {
        let v = vec![1.0f32, 2.0, 3.0];
        let score = cosine(&v, &v);
        assert!((score - 1.0).abs() < 1e-6, "identical vectors should score 1.0");
    }

    #[test]
    fn cosine_orthogonal_vectors() {
        let a = vec![1.0f32, 0.0];
        let b = vec![0.0f32, 1.0];
        let score = cosine(&a, &b);
        assert!(score.abs() < 1e-6, "orthogonal vectors should score 0.0");
    }

    #[test]
    fn cosine_zero_vector() {
        let a = vec![0.0f32, 0.0];
        let b = vec![1.0f32, 2.0];
        assert_eq!(cosine(&a, &b), 0.0);
    }

    #[test]
    fn chunk_text_splits_large_content() {
        let text = "a".repeat(4000);
        let chunks = chunk_text(&text, 1800, 200);
        assert!(chunks.len() >= 2);
        for c in &chunks {
            assert!(c.len() <= 1800);
        }
    }

    #[test]
    fn chunk_text_empty_input() {
        assert!(chunk_text("", 1800, 200).is_empty());
    }

    #[test]
    fn chunk_text_smaller_than_chunk_size() {
        let text = "hello world";
        let chunks = chunk_text(text, 1800, 200);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }
}
