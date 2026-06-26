//! Loads Kiwi plugins from native dynamic libraries.
//!
//! This crate isolates `unsafe` FFI required by `libloading` so the main Kiwi
//! binary can remain `unsafe`-free.

mod decode;
mod error;
mod host;

pub use error::LoadError;
pub use host::{LoadedPlugin, PluginHost};

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use kiwi_plugin_api::{PluginApi, PluginCommand, PluginInitFn, PluginRegistrar, PluginResult};
use libloading::{Library, Symbol};

use crate::decode::decode_static_str;

struct CommandCollector {
    commands: Vec<PluginCommand>,
}

impl PluginRegistrar for CommandCollector {
    fn register_command(&mut self, command: PluginCommand) {
        self.commands.push(command);
    }
}

/// Load one plugin library and collect registered commands.
///
/// `kiwi_version` is checked against `min_kiwi_version` by the caller.
pub fn load_plugin(library_path: &Path, entry_symbol: &str) -> Result<LoadedPlugin, LoadError> {
    // SAFETY: Loading a plugin path provided by the user's plugins directory.
    let library = unsafe { Library::new(library_path) }.map_err(|source| LoadError::Library {
        path: library_path.to_path_buf(),
        source,
    })?;

    let descriptor = {
        let init = resolve_init(&library, entry_symbol)?;
        init()
    };

    let name = decode_static_str(descriptor.name)?;
    let version = decode_static_str(descriptor.version)?;

    let mut collector = CommandCollector {
        commands: Vec::new(),
    };
    let mut api = PluginApi::new(&mut collector);
    let register_result = catch_unwind(AssertUnwindSafe(|| (descriptor.register)(&mut api)));
    if register_result.is_err() {
        return Err(LoadError::RegisterPanicked {
            plugin: name.clone(),
            library: library_path.to_path_buf(),
        });
    }

    Ok(LoadedPlugin {
        name,
        version,
        commands: collector.commands,
        library,
    })
}

fn resolve_init<'lib>(
    library: &'lib Library,
    entry_symbol: &str,
) -> Result<Symbol<'lib, PluginInitFn>, LoadError> {
    // SAFETY: Symbol type matches the plugin entry point contract.
    let init = unsafe {
        library
            .get::<PluginInitFn>(entry_symbol.as_bytes())
            .map_err(|source| LoadError::Symbol {
                symbol: entry_symbol.to_string(),
                source,
            })?
    };
    Ok(init)
}

/// Invoke a plugin command callback, mapping panics to `PluginResult::Err`.
pub fn invoke_plugin_command(callback: extern "C" fn() -> PluginResult) -> PluginInvokeOutcome {
    match catch_unwind(AssertUnwindSafe(|| callback())) {
        Ok(result) => PluginInvokeOutcome::Completed(result),
        Err(_) => PluginInvokeOutcome::Panicked,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginInvokeOutcome {
    Completed(PluginResult),
    Panicked,
}

#[cfg(test)]
mod tests {
    use kiwi_plugin_api::StaticStr;

    use super::*;

    #[test]
    fn decode_static_str_round_trip() {
        let value = StaticStr::from_static("hello");
        assert_eq!(decode_static_str(value).expect("decode"), "hello");
    }

    #[test]
    fn decode_rejects_null_pointer() {
        let value = StaticStr {
            ptr: std::ptr::null(),
            len: 0,
        };
        assert!(decode_static_str(value).is_err());
    }

    #[test]
    fn load_missing_library_returns_error() {
        let err = load_plugin(Path::new("/no/such/plugin.so"), "kiwi_plugin_init")
            .expect_err("missing library");
        assert!(matches!(err, LoadError::Library { .. }));
    }
}
