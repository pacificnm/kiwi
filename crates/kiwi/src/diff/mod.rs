mod generate;
mod io;
mod types;

pub use io::spawn_file_diff_load;
pub use types::{DiffLine, DiffLineKind, DiffSource, FileDiffLoadResult};
