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

/// Walk `source`, chunk eligible files, embed, and upsert into knowledge_base.
///
/// `extensions` filters by file extension (case-insensitive, no leading dot).
/// An empty slice means "accept all files that parse as UTF-8".
/// HTML/HTM files are stripped of tags before chunking regardless of how they
/// were selected.
pub fn index_knowledge(
    collection: &str,
    source: &Path,
    db: &mut crate::db::MemoryDb,
    embed: &EmbedClient,
    extensions: &[&str],
) -> Result<()> {
    eprintln!("indexing knowledge collection '{collection}' from {}", source.display());

    let mut indexed = 0usize;
    let mut skipped = 0usize;
    let mut files_seen = 0usize;

    for entry in WalkDir::new(source)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Extension filter
        if !extensions.is_empty() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !extensions.iter().any(|&ex| ex.eq_ignore_ascii_case(&ext)) {
                continue;
            }
        }

        files_seen += 1;

        let raw = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue, // binary or unreadable — skip silently
        };

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let content = if ext == "html" || ext == "htm" {
            strip_html(&raw)
        } else {
            raw
        };

        let content = content.trim().to_string();
        if content.is_empty() {
            continue;
        }

        let rel = path
            .strip_prefix(source)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();

        let chunks = chunk_text(&content);
        for chunk in chunks {
            let hash = sha256_hex(&chunk);
            match embed.embed(&chunk) {
                Ok(embedding) => {
                    db.upsert_knowledge(collection, &rel, &chunk, &hash, &embedding)?;
                    indexed += 1;
                }
                Err(e) => {
                    eprintln!("  embed error for {rel}: {e}");
                    skipped += 1;
                }
            }
        }
    }

    eprintln!(
        "done — {files_seen} files, {indexed} chunks indexed, {skipped} skipped"
    );
    Ok(())
}

/// Remove HTML tags and decode common entities, collapsing whitespace runs.
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                // Replace block-level tags with a newline for readability
                out.push(' ');
            }
            '&' if !in_tag => {
                // Collect until ';' or whitespace for entity decoding
                let mut entity = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ';' {
                        chars.next();
                        break;
                    }
                    if c.is_whitespace() {
                        break;
                    }
                    entity.push(chars.next().unwrap());
                }
                let decoded = match entity.as_str() {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "nbsp" | "#160" => " ",
                    "quot" | "#34" => "\"",
                    "#39" | "apos" => "'",
                    "ndash" | "#8211" => "–",
                    "mdash" | "#8212" => "—",
                    _ => {
                        out.push('&');
                        out.push_str(&entity);
                        continue;
                    }
                };
                out.push_str(decoded);
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }

    // Collapse runs of whitespace (but preserve single newlines)
    let mut result = String::with_capacity(out.len());
    let mut last_was_space = false;
    for ch in out.chars() {
        if ch == '\n' || ch == '\r' {
            if !last_was_space {
                result.push('\n');
            }
            last_was_space = true;
        } else if ch.is_whitespace() {
            if !last_was_space {
                result.push(' ');
            }
            last_was_space = true;
        } else {
            result.push(ch);
            last_was_space = false;
        }
    }
    result
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
