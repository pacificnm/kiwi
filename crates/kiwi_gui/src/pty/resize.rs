//! PTY dimension helpers (SPEC-002 / Phase 4).

use super::{DEFAULT_PTY_COLS, DEFAULT_PTY_ROWS};

const MIN_PTY_COLS: u16 = 2;
const MIN_PTY_ROWS: u16 = 1;

#[must_use]
pub fn effective_pty_size(cols: u16, rows: u16) -> (u16, u16) {
    (
        if cols == 0 {
            DEFAULT_PTY_COLS
        } else {
            cols.max(MIN_PTY_COLS)
        },
        if rows == 0 {
            DEFAULT_PTY_ROWS
        } else {
            rows.max(MIN_PTY_ROWS)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_pty_size_uses_defaults_when_unmeasured() {
        assert_eq!(effective_pty_size(0, 0), (DEFAULT_PTY_COLS, DEFAULT_PTY_ROWS));
    }

    #[test]
    fn effective_pty_size_clamps_to_minimum() {
        assert_eq!(effective_pty_size(1, 1), (MIN_PTY_COLS, MIN_PTY_ROWS));
    }
}
