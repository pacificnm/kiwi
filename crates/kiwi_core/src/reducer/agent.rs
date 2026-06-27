
use crate::agent::infer_status_from_scrollback;
use crate::navigation::MainTab;
use crate::state::ReduceView;

use crate::events::{AgentEffect, SideEffect};

pub fn agent_spawn_effects_if_needed(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    agent_spawn_effects_for(state, state.agent_manager.active_id())
}

pub fn agent_new_effects(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    reduce_agent_new(state)
}

pub fn agent_cycle_effects(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    reduce_agent_cycle(state, delta)
}

pub(super) fn agent_spawn_effects_for(
    state: &mut ReduceView<'_>,
    id: crate::agent::AgentId,
) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    let needs_spawn = state.agent_manager.pty(id).is_some_and(|pty| !pty.spawned);
    if !needs_spawn {
        return Vec::new();
    }

    state.set_dirty();
    vec![SideEffect::Agent(AgentEffect::Spawn(id))]
}

pub(super) fn reduce_agent_output(
    state: &mut ReduceView<'_>,
    agent_id: crate::agent::AgentId,
    data: Vec<u8>,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };

    let cols = pty.cols;
    pty.scrollback.set_cols(cols);
    pty.scrollback.append_bytes(&data);
    if let Some(status) = infer_status_from_scrollback(&pty.scrollback) {
        pty.status = status;
    }
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_agent_exited(
    state: &mut ReduceView<'_>,
    agent_id: crate::agent::AgentId,
    code: i32,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };

    pty.apply_exit(code);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_agent_new(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    let linked_issue = state.github.selected_issue;
    match state.agent_manager.create_agent(None, linked_issue) {
        Ok(id) => {
            state.set_dirty();
            vec![SideEffect::Agent(AgentEffect::Spawn(id))]
        }
        Err(crate::agent::AgentManagerError::AtCapacity) => {
            state
                .notifications
                .show_toast("Agent limit reached (max 3 sessions)");
            Vec::new()
        }
        Err(_) => Vec::new(),
    }
}

pub(super) fn reduce_agent_set_active(
    state: &mut ReduceView<'_>,
    id: crate::agent::AgentId,
) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    if state.agent_manager.set_active(id).is_err() {
        return Vec::new();
    }

    state.set_dirty();
    agent_spawn_effects_for(state, id)
}

pub(super) fn reduce_agent_cycle(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    if state.agent_manager.session_count() <= 1 {
        return Vec::new();
    }

    let id = state.agent_manager.cycle_active(delta);
    state.set_dirty();
    agent_spawn_effects_for(state, id)
}

pub(super) fn reduce_agent_restart(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    state.set_dirty();
    vec![SideEffect::Agent(AgentEffect::Restart(state.agent_manager.active_id()))]
}

pub(super) fn reduce_agent_scroll(state: &mut ReduceView<'_>, delta: i32) -> Vec<SideEffect> {
    if delta == 0 {
        return Vec::new();
    }

    let page_size = state.viewport.agent_rows;
    state.active_agent_mut().scroll_by(delta, page_size);
    state.set_dirty();
    Vec::new()
}

pub(super) fn reduce_agent_scroll_lines(state: &mut ReduceView<'_>, lines: i32) -> Vec<SideEffect> {
    if lines == 0 {
        return Vec::new();
    }

    let page_size = state.viewport.agent_rows;
    state.active_agent_mut().scroll_by_lines(lines, page_size);
    state.set_dirty();
    Vec::new()
}
