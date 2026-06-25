use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::time::SystemTime;

pub const BINARY_SAMPLE_BYTES: usize = 8_192;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewLoadResult {
    pub lines: Vec<String>,
    pub truncated: bool,
    pub oversize: bool,
    pub binary: bool,
    pub lossy_utf8: bool,
    pub file_size: u64,
    pub modified_at: Option<SystemTime>,
    pub error: Option<String>,
}

impl PreviewLoadResult {
    fn error(message: String) -> Self {
        Self {
            lines: Vec::new(),
            truncated: false,
            oversize: false,
            binary: false,
            lossy_utf8: false,
            file_size: 0,
            modified_at: None,
            error: Some(message),
        }
    }
}

pub fn load_preview_file(path: &Path, max_size_bytes: u64) -> PreviewLoadResult {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) => return PreviewLoadResult::error(err.to_string()),
    };

    if metadata.is_dir() {
        return PreviewLoadResult::error(format!("{} is a directory", path.display()));
    }

    let file_size = metadata.len();
    let modified_at = metadata.modified().ok();
    if file_size > max_size_bytes {
        return PreviewLoadResult {
            lines: Vec::new(),
            truncated: false,
            oversize: true,
            binary: false,
            lossy_utf8: false,
            file_size,
            modified_at,
            error: Some(format!(
                "File too large to preview ({file_size} bytes > {max_size_bytes} byte limit)"
            )),
        };
    }

    if is_likely_binary(path, file_size) {
        return PreviewLoadResult {
            lines: Vec::new(),
            truncated: false,
            oversize: false,
            binary: true,
            lossy_utf8: false,
            file_size,
            modified_at,
            error: None,
        };
    }

    read_text_lines(path, file_size, max_size_bytes, modified_at)
}

fn is_likely_binary(path: &Path, file_size: u64) -> bool {
    let sample_len = file_size.min(BINARY_SAMPLE_BYTES as u64) as usize;
    if sample_len == 0 {
        return false;
    }

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };
    let mut sample = vec![0_u8; sample_len];
    if file.read_exact(&mut sample).is_err() {
        return false;
    }
    sample.contains(&0)
}

fn read_text_lines(
    path: &Path,
    file_size: u64,
    max_size_bytes: u64,
    modified_at: Option<SystemTime>,
) -> PreviewLoadResult {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return PreviewLoadResult::error(err.to_string()),
    };

    let mut reader = BufReader::new(file);
    let mut lines = Vec::new();
    let mut consumed = 0_u64;
    let mut lossy_utf8 = false;
    let mut truncated = false;

    loop {
        let mut buffer = Vec::new();
        match reader.read_until(b'\n', &mut buffer) {
            Ok(0) => break,
            Ok(_) => {}
            Err(err) => return PreviewLoadResult::error(err.to_string()),
        }

        consumed += u64::try_from(buffer.len()).unwrap_or(u64::MAX);
        if consumed > max_size_bytes {
            truncated = true;
            break;
        }

        if buffer.ends_with(b"\n") {
            buffer.pop();
            if buffer.ends_with(b"\r") {
                buffer.pop();
            }
        }

        let (line, used_lossy) = decode_line(&buffer);
        lossy_utf8 |= used_lossy;
        lines.push(line);
    }

    PreviewLoadResult {
        lines,
        truncated,
        oversize: false,
        binary: false,
        lossy_utf8,
        file_size,
        modified_at,
        error: None,
    }
}

fn decode_line(bytes: &[u8]) -> (String, bool) {
    match std::str::from_utf8(bytes) {
        Ok(text) => (text.to_string(), false),
        Err(_) => (String::from_utf8_lossy(bytes).into_owned(), true),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn load_preview_file_reads_text_lines() {
        let temp = std::env::temp_dir().join(format!("kiwi-preview-text-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        fs::write(temp.join("sample.txt"), "alpha\nbeta\n").expect("write");

        let result = load_preview_file(&temp.join("sample.txt"), 1_048_576);
        assert!(result.error.is_none());
        assert_eq!(result.lines, vec!["alpha".to_string(), "beta".to_string()]);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn load_preview_file_detects_binary() {
        let temp = std::env::temp_dir().join(format!("kiwi-preview-bin-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        fs::write(temp.join("data.bin"), [b'h', b'i', 0, b'!', b'\n']).expect("write");

        let result = load_preview_file(&temp.join("data.bin"), 1_048_576);
        assert!(result.binary);
        assert!(result.lines.is_empty());

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn load_preview_file_rejects_oversize_without_reading() {
        let temp = std::env::temp_dir().join(format!("kiwi-preview-big-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        fs::write(temp.join("big.txt"), vec![b'a'; 2048]).expect("write");

        let result = load_preview_file(&temp.join("big.txt"), 1024);
        assert!(result.oversize);
        assert!(result.lines.is_empty());
        assert_eq!(result.file_size, 2048);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn load_preview_file_marks_lossy_utf8() {
        let temp = std::env::temp_dir().join(format!("kiwi-preview-utf8-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("mkdir");
        fs::write(temp.join("bad.txt"), [0xFF, b'\n']).expect("write");

        let result = load_preview_file(&temp.join("bad.txt"), 1_048_576);
        assert!(result.lossy_utf8);
        assert_eq!(result.lines.len(), 1);

        let _ = fs::remove_dir_all(temp);
    }
}
