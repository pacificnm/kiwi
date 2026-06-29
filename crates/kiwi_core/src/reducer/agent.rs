
use crate::agent::{infer_status_from_scrollback, AgentId, ChatSession, ToolUse, ToolResult};
use crate::navigation::MainTab;
use crate::state::ReduceView;

use crate::events::{AgentEffect, SideEffect};

pub fn agent_spawn_effects_if_needed(state: &mut ReduceView<'_>) -> Vec<SideEffect> {
    if state.navigation.main_tab != MainTab::Agent {
        return Vec::new();
    }

    refresh_active_agent_status_heuristic(state);
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
    let refresh_manager = {
        let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
            return Vec::new();
        };

        let cols = pty.cols;
        pty.scrollback.set_cols(cols);
        pty.scrollback.append_bytes(&data);

        if state.navigation.main_tab == MainTab::Agent {
            pty.status_check_accum += data.len();
            if pty.status_check_accum >= 512 {
                pty.status_check_accum = 0;
                apply_status_heuristic(pty)
            } else {
                false
            }
        } else {
            false
        }
    };
    if refresh_manager {
        state.agent_manager.refresh_status_label();
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
    state.agent_manager.refresh_status_label();
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

fn refresh_active_agent_status_heuristic(state: &mut ReduceView<'_>) {
    let id = state.agent_manager.active_id();
    let mut refresh_manager = false;
    if let Some(pty) = state.agent_manager.pty_mut(id) {
        pty.status_check_accum = 0;
        refresh_manager = apply_status_heuristic(pty);
    }
    if refresh_manager {
        state.agent_manager.refresh_status_label();
    }
}

fn apply_status_heuristic(pty: &mut crate::state::AgentState) -> bool {
    if let Some(status) = infer_status_from_scrollback(&pty.scrollback) {
        if pty.status != status {
            pty.status = status;
            pty.refresh_status_bar_label();
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Native chat reducers — wired in Phase 1, streaming logic active in Phase 2.
// ---------------------------------------------------------------------------

pub(super) fn reduce_agent_user_send(
    state: &mut ReduceView<'_>,
    agent_id: AgentId,
    text: String,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };
    let chat = pty.chat.get_or_insert_with(ChatSession::default);
    chat.append_user_message(text);
    chat.is_streaming = true;
    state.set_dirty();
    vec![SideEffect::Agent(AgentEffect::StreamRequest(agent_id))]
}

pub(super) fn reduce_agent_token_chunk(
    state: &mut ReduceView<'_>,
    agent_id: AgentId,
    text: String,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };
    if let Some(chat) = &mut pty.chat {
        chat.streaming_text.push_str(&text);
        state.set_dirty();
    }
    Vec::new()
}

pub(super) fn reduce_agent_tool_call_start(
    state: &mut ReduceView<'_>,
    agent_id: AgentId,
    tool_use_id: String,
    tool_name: String,
    input_json: String,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };
    if let Some(chat) = &mut pty.chat {
        let tool = ToolUse::new(tool_use_id.clone(), tool_name.clone(), input_json.clone());
        chat.active_tool_call = Some(tool.clone());
        chat.append_tool_use(tool);
        state.set_dirty();
        return vec![SideEffect::Agent(AgentEffect::ExecuteTool {
            agent_id,
            tool_use_id,
            tool_name,
            input_json,
        })];
    }
    Vec::new()
}

pub(super) fn reduce_agent_tool_result(
    state: &mut ReduceView<'_>,
    agent_id: AgentId,
    tool_use_id: String,
    content: String,
    is_error: bool,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };
    if let Some(chat) = &mut pty.chat {
        chat.active_tool_call = None;
        let result = if is_error {
            ToolResult::error(tool_use_id, content)
        } else {
            ToolResult::ok(tool_use_id, content)
        };
        chat.append_tool_result(result);
        state.set_dirty();
        return vec![SideEffect::Agent(AgentEffect::StreamRequest(agent_id))];
    }
    Vec::new()
}

pub(super) fn reduce_agent_turn_complete(
    state: &mut ReduceView<'_>,
    agent_id: AgentId,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };
    if let Some(chat) = &mut pty.chat {
        chat.flush_streaming_turn();
        chat.status = crate::agent::AgentStatus::Idle;
        chat.refresh_status_bar_label();
        state.agent_manager.refresh_status_label();
        state.set_dirty();
    }
    Vec::new()
}

pub(super) fn reduce_agent_api_error(
    state: &mut ReduceView<'_>,
    agent_id: AgentId,
    message: String,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };
    if let Some(chat) = &mut pty.chat {
        chat.is_streaming = false;
        chat.error = Some(message);
        chat.status = crate::agent::AgentStatus::Error;
        chat.refresh_status_bar_label();
        state.agent_manager.refresh_status_label();
        state.set_dirty();
    }
    Vec::new()
}

pub(super) fn reduce_agent_toggle_tool_expand(
    state: &mut ReduceView<'_>,
    agent_id: AgentId,
    tool_use_id: String,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };
    if let Some(chat) = &mut pty.chat {
        chat.toggle_tool_expand(&tool_use_id);
        state.set_dirty();
    }
    Vec::new()
}

pub(super) fn reduce_agent_clear_history(
    state: &mut ReduceView<'_>,
    agent_id: AgentId,
) -> Vec<SideEffect> {
    let Some(pty) = state.agent_manager.pty_mut(agent_id) else {
        return Vec::new();
    };
    if let Some(chat) = &mut pty.chat {
        chat.clear();
        state.agent_manager.refresh_status_label();
        state.set_dirty();
    }
    Vec::new()
}
