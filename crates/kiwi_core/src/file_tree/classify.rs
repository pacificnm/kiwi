use crate::theme::SemanticRole;

use super::FileNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTypeCategory {
    Dir,
    Source,
    Script,
    Markup,
    Config,
    Data,
    Media,
    Other,
}

impl FileTypeCategory {
    #[must_use]
    pub const fn semantic_role(self) -> SemanticRole {
        match self {
            Self::Dir => SemanticRole::FileDir,
            Self::Source => SemanticRole::FileSource,
            Self::Script => SemanticRole::FileScript,
            Self::Markup => SemanticRole::FileMarkup,
            Self::Config => SemanticRole::FileConfig,
            Self::Data => SemanticRole::FileData,
            Self::Media => SemanticRole::FileMedia,
            Self::Other => SemanticRole::FileOther,
        }
    }
}

#[must_use]
pub fn file_type_category(node: &FileNode) -> FileTypeCategory {
    if node.is_dir {
        return FileTypeCategory::Dir;
    }

    if let Some(category) = category_for_name(&node.name) {
        return category;
    }

    extension_category(node.path.extension().and_then(|ext| ext.to_str()))
        .unwrap_or(FileTypeCategory::Other)
}

fn category_for_name(name: &str) -> Option<FileTypeCategory> {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "dockerfile" | "containerfile" | "makefile" | "gnumakefile" | "cmakelists.txt" => {
            Some(FileTypeCategory::Config)
        }
        "readme" | "license" | "changelog" | "copying" | "authors" | "contributing" => {
            Some(FileTypeCategory::Markup)
        }
        _ => None,
    }
}

fn extension_category(ext: Option<&str>) -> Option<FileTypeCategory> {
    let ext = ext?.to_ascii_lowercase();
    let category = match ext.as_str() {
        "rs" | "c" | "cc" | "cpp" | "cxx" | "h" | "hpp" | "hh" | "go" | "java" | "kt" | "kts"
        | "swift" | "cs" | "fs" | "fsx" | "zig" | "v" | "dart" | "scala" | "clj" | "cljs"
        | "elm" | "ex" | "exs" | "erl" | "hrl" | "hs" | "ml" | "mli" | "r" | "jl" | "asm" | "s"
        | "m" | "mm" | "nim" | "cr" | "d" | "pas" | "vb" | "groovy" | "cu" => {
            FileTypeCategory::Source
        }
        "py" | "pyw" | "pyi" | "sh" | "bash" | "zsh" | "fish" | "rb" | "pl" | "pm" | "lua"
        | "js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx" | "vue" | "svelte" | "php" | "ps1"
        | "awk" | "tcl" => FileTypeCategory::Script,
        "md" | "mdx" | "rst" | "tex" | "org" | "txt" | "adoc" | "wiki" | "rtf" => {
            FileTypeCategory::Markup
        }
        "toml" | "yaml" | "yml" | "json" | "jsonc" | "ini" | "conf" | "cfg" | "properties"
        | "env" | "editorconfig" | "gitignore" | "gitattributes" | "npmrc" | "nvmrc" => {
            FileTypeCategory::Config
        }
        "sql" | "csv" | "tsv" | "xml" | "db" | "sqlite" | "parquet" | "lock" => {
            FileTypeCategory::Data
        }
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp" | "tiff" | "tif"
        | "woff" | "woff2" | "ttf" | "otf" | "eot" | "mp3" | "mp4" | "wav" | "pdf" | "mpg"
        | "mpeg" | "mov" | "avi" | "mkv" => FileTypeCategory::Media,
        _ => return None,
    };
    Some(category)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn file_node(name: &str, is_dir: bool) -> FileNode {
        FileNode {
            path: PathBuf::from(name),
            name: name.to_string(),
            is_dir,
            expanded: false,
            children_loaded: false,
            git_status: None,
            load_error: None,
        }
    }

    #[test]
    fn directories_map_to_dir_category() {
        assert_eq!(
            file_type_category(&file_node("src", true)),
            FileTypeCategory::Dir
        );
    }

    #[test]
    fn extensions_classify_common_types() {
        assert_eq!(
            file_type_category(&file_node("main.rs", false)),
            FileTypeCategory::Source
        );
        assert_eq!(
            file_type_category(&file_node("app.py", false)),
            FileTypeCategory::Script
        );
        assert_eq!(
            file_type_category(&file_node("README.md", false)),
            FileTypeCategory::Markup
        );
        assert_eq!(
            file_type_category(&file_node("Cargo.toml", false)),
            FileTypeCategory::Config
        );
        assert_eq!(
            file_type_category(&file_node("schema.sql", false)),
            FileTypeCategory::Data
        );
        assert_eq!(
            file_type_category(&file_node("logo.png", false)),
            FileTypeCategory::Media
        );
        assert_eq!(
            file_type_category(&file_node("notes", false)),
            FileTypeCategory::Other
        );
    }

    #[test]
    fn special_filenames_classify_without_extension() {
        assert_eq!(
            file_type_category(&file_node("Dockerfile", false)),
            FileTypeCategory::Config
        );
        assert_eq!(
            file_type_category(&file_node("LICENSE", false)),
            FileTypeCategory::Markup
        );
    }

    #[test]
    fn semantic_roles_map_to_theme_keys() {
        assert_eq!(
            FileTypeCategory::Source.semantic_role(),
            SemanticRole::FileSource
        );
        assert_eq!(
            FileTypeCategory::Other.semantic_role(),
            SemanticRole::FileOther
        );
    }
}
