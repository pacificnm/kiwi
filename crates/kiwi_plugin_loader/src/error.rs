use std::path::PathBuf;

#[derive(Debug)]
pub enum LoadError {
    Library {
        path: PathBuf,
        source: libloading::Error,
    },
    Symbol {
        symbol: String,
        source: libloading::Error,
    },
    InvalidDescriptor(&'static str),
    RegisterPanicked {
        plugin: String,
        library: PathBuf,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Library { path, source } => {
                write!(
                    f,
                    "failed to load plugin library {}: {source}",
                    path.display()
                )
            }
            Self::Symbol { symbol, source } => {
                write!(f, "plugin missing symbol `{symbol}`: {source}")
            }
            Self::InvalidDescriptor(reason) => write!(f, "invalid plugin descriptor: {reason}"),
            Self::RegisterPanicked { plugin, library } => write!(
                f,
                "plugin `{plugin}` panicked during registration ({})",
                library.display()
            ),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Library { source, .. } | Self::Symbol { source, .. } => Some(source),
            _ => None,
        }
    }
}
