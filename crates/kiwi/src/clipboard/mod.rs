mod io;
mod keys;
mod paste;
mod target;

pub use io::ClipboardService;
pub use keys::{clipboard_op_from_key, clipboard_shortcut_allowed, ClipboardOp};
pub use kiwi_core::clipboard::PasteTarget;
pub use paste::pty_paste_bytes;
pub use target::{resolve_copy_text, resolve_paste_target};
