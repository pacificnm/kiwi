use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

use crate::db::MemoryDb;
use crate::embed::EmbedClient;

const CHUNK_SIZE: usize = 1800;
const CHUNK_OVERLAP: usize = 200;

/// File paths/patterns that mirror what the Python index_memory.py indexes.
const ROOT_FILES: &[&str] = &[
    "README.md",
    "AGENTS.md",
    "BUILD_COMMANDS.md",
    "KNOWN_ISSUES.md",
    "CONTRIBUTING.md",
    "CODE_OF_CONDUCT.md",
    "SECURITY.md",
    "LICENSE.md",
    "plan.md",
];

const INDEX_DIRS: &[&str] = &["docs", "tools"];

/// Walk the project root, chunk eligible files, embed, and upsert into project_memory.
pub fn index_project(root: &Path, db: &mut MemoryDb, embed: &EmbedClient) -> Result<()> {
    let mut files: Vec<std::path::PathBuf> = Vec::new();

    // Collect root-level markdown/text files
    for name in ROOT_FILES {
        let p = root.join(name);
        if p.exists() {
            files.push(p);
        }
    }

    // Walk designated directories
    for dir in INDEX_DIRS {
        let dir_path = root.join(dir);
        if !dir_path.exists() {
            continue;
        }
        for entry in WalkDir::new(&dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            files.push(entry.into_path());
        }
    }

    let total = files.len();
    eprintln!("indexing {total} files into project_memory…");

    let mut indexed = 0usize;
    let mut skipped = 0usize;

    for path in &files {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("  skip (unreadable): {}", path.display());
                continue;
            }
        };

        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();

        let chunks = chunk_text(&content);
        for chunk in chunks {
            let hash = sha256_hex(&chunk);
            match embed.embed(&chunk) {
                Ok(embedding) => {
                    db.upsert(&rel, &chunk, &hash, &embedding)?;
                    indexed += 1;
                }
                Err(e) => {
                    eprintln!("  embed error for {rel}: {e}");
                    skipped += 1;
                }
            }
        }
    }

    eprintln!("done — {indexed} chunks indexed, {skipped} skipped");
    Ok(())
}

fn chunk_text(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let len = text.len();
    if len == 0 {
        return chunks;
    }
    let mut start = 0usize;
    while start < len {
        let raw_end = (start + CHUNK_SIZE).min(len);
        let end = (raw_end..=len)
            .find(|&i| text.is_char_boundary(i))
            .unwrap_or(len);
        chunks.push(text[start..end].to_string());
        if end >= len {
            break;
        }
        let raw_next = end.saturating_sub(CHUNK_OVERLAP);
        start = (raw_next..=end)
            .find(|&i| text.is_char_boundary(i))
            .unwrap_or(end);
    }
    chunks
}

fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher
        .finalize()
        .iter()
        .fold(String::new(), |mut s, b| {
            use std::fmt::Write;
            let _ = write!(s, "{b:02x}");
            s
        })
}
