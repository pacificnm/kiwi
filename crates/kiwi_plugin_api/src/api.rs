/// Result returned by plugin command callbacks.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginResult {
    Ok = 0,
    Err = 1,
}

/// A command registered by a plugin for the Kiwi command palette.
#[derive(Debug, Clone)]
pub struct PluginCommand {
    pub id: String,
    pub title: String,
    pub callback: extern "C" fn() -> PluginResult,
}

impl PartialEq for PluginCommand {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.title == other.title
    }
}

impl Eq for PluginCommand {}

/// Host-side hook for collecting plugin registrations.
pub trait PluginRegistrar {
    fn register_command(&mut self, command: PluginCommand);
}

/// Registration surface passed to plugins during initialization.
pub struct PluginApi<'a> {
    registrar: &'a mut dyn PluginRegistrar,
}

impl<'a> PluginApi<'a> {
    pub fn new(registrar: &'a mut dyn PluginRegistrar) -> Self {
        Self { registrar }
    }

    /// Register a palette command.
    ///
    /// `id` should be unique across all plugins, e.g. `my-plugin.hello`.
    pub fn register_command(
        &mut self,
        id: impl Into<String>,
        title: impl Into<String>,
        callback: extern "C" fn() -> PluginResult,
    ) {
        self.registrar.register_command(PluginCommand {
            id: id.into(),
            title: title.into(),
            callback,
        });
    }
}

/// Static descriptor returned by a plugin's [`PLUGIN_INIT_SYMBOL`](crate::PLUGIN_INIT_SYMBOL) entry point.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PluginDescriptor {
    pub name: StaticStr,
    pub version: StaticStr,
    pub register: PluginRegisterFn,
}

/// UTF-8 string slice passed across the plugin ABI.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticStr {
    pub ptr: *const u8,
    pub len: usize,
}

impl StaticStr {
    #[must_use]
    pub const fn from_static(value: &'static str) -> Self {
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

/// Plugin initialization function resolved from a loaded dynamic library.
///
/// Plugins export this as `extern "C" fn kiwi_plugin_init() -> PluginDescriptor`; the
/// host resolves the symbol and invokes it with Rust calling convention.
pub type PluginInitFn = fn() -> PluginDescriptor;

/// Registration callback invoked by Kiwi into plugin code.
pub type PluginRegisterFn = fn(&mut PluginApi<'_>);

/// Ergonomic trait for plugin authors; use [`declare_plugin!`] to export the entry symbol.
pub trait KiwiPlugin {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn register(&self, api: &mut PluginApi<'_>);
}

/// Export [`kiwi_plugin_init`](crate::PLUGIN_INIT_SYMBOL) for a type implementing [`KiwiPlugin`].
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty) => {
        fn _kiwi_plugin_register(api: &mut $crate::PluginApi<'_>) {
            <$plugin_type as $crate::KiwiPlugin>::register(
                &<$plugin_type as ::std::default::Default>::default(),
                api,
            );
        }

        #[cfg_attr(not(test), no_mangle)]
        #[allow(improper_ctypes_definitions)]
        pub extern "C" fn kiwi_plugin_init() -> $crate::PluginDescriptor {
            let plugin = <$plugin_type as ::std::default::Default>::default();
            $crate::PluginDescriptor {
                name: $crate::StaticStr::from_static(<$plugin_type as $crate::KiwiPlugin>::name(
                    &plugin,
                )),
                version: $crate::StaticStr::from_static(
                    <$plugin_type as $crate::KiwiPlugin>::version(&plugin),
                ),
                register: _kiwi_plugin_register,
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RecordingRegistrar {
        commands: Vec<PluginCommand>,
    }

    impl PluginRegistrar for RecordingRegistrar {
        fn register_command(&mut self, command: PluginCommand) {
            self.commands.push(command);
        }
    }

    extern "C" fn hello_callback() -> PluginResult {
        PluginResult::Ok
    }

    #[derive(Default)]
    struct HelloPlugin;

    impl KiwiPlugin for HelloPlugin {
        fn name(&self) -> &'static str {
            "hello"
        }

        fn version(&self) -> &'static str {
            "0.1.0"
        }

        fn register(&self, api: &mut PluginApi<'_>) {
            api.register_command("hello.greet", "Hello: Greet", hello_callback);
        }
    }

    #[test]
    fn plugin_api_registers_commands() {
        let mut registrar = RecordingRegistrar { commands: vec![] };
        let mut api = PluginApi::new(&mut registrar);
        HelloPlugin.register(&mut api);
        assert_eq!(registrar.commands.len(), 1);
        assert_eq!(registrar.commands[0].id, "hello.greet");
        assert_eq!(registrar.commands[0].title, "Hello: Greet");
    }

    #[test]
    fn declare_plugin_macro_exports_descriptor() {
        declare_plugin!(HelloPlugin);
        let descriptor = kiwi_plugin_init();
        assert_eq!(descriptor.name, StaticStr::from_static("hello"));
        assert_eq!(descriptor.version, StaticStr::from_static("0.1.0"));

        let mut registrar = RecordingRegistrar { commands: vec![] };
        let mut api = PluginApi::new(&mut registrar);
        (descriptor.register)(&mut api);
        assert_eq!(registrar.commands[0].id, "hello.greet");
    }
}
