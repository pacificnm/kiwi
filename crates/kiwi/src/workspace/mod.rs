mod persistence;

pub const MAX_PALETTE_HISTORY_ENTRIES: usize = 50;

pub use persistence::{load_palette_history, save_palette_history};
