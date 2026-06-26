//! TUI-facing shell API; domain logic lives in [`kiwi_core::shell`].

mod keys;

pub use keys::encode_key;
#[allow(unused_imports)]
pub use kiwi_core::shell::{
    shell_launch_spec, ScrollbackBuffer, ShellError, ShellLaunchSpec, ShellOutputReader,
    ShellSession,
};
