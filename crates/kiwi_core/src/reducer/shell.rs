
use crate::state::ReduceView;

use crate::events::SideEffect;

pub(super) fn reduce_shell_output(state: &mut ReduceView<'_>, data: Vec<u8>) -> Vec<SideEffect> {
    state.shell.scrollback.set_cols(state.shell.cols);
    state.shell.scrollback.append_bytes(&data);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_shell_exited(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    state.shell.running = false;
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_shell_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let page_size = state.viewport.shell_rows;
    state.shell.scroll_by(delta, page_size);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_shell_scroll_lines(state: &mut ReduceView<'_>, lines: i32) -> Vec<SideEffect> {
    if lines == 0 {
        return Vec::new();
    }

    let page_size = state.viewport.shell_rows;
    state.shell.scroll_by_lines(lines, page_size);
    state.set_dirty();
    Vec::new()
}
