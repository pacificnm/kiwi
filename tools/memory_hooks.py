"""Cursor hook handlers for Kiwi memory-first and pre-compaction context saves."""

from __future__ import annotations

import json
import sys

from context_memory import format_entry, list_context
from transcript_snapshot import git_branch, snapshot_transcript


MEMORY_FIRST_CONTEXT = """## Kiwi memory requirements (mandatory)

Before reading or editing code, or running implementation commands:

1. **Project memory** — call `search_project_memory` for the affected subsystem,
   milestone, SPEC/ADR numbers, and known issues.
2. **Context memory** — call `search_context_memory` (and `list_context_memory`
   when resuming work) using `session_key="{session_key}"` when available.

After major checkpoints, before handoff, or when context is getting full, call
`save_context_memory` with decisions, file paths, blockers, and verification results.

A pre-compaction hook also snapshots the transcript automatically; agent-initiated
saves remain required at stable checkpoints.
"""


def read_hook_input() -> dict:
    raw = sys.stdin.read()
    if not raw.strip():
        return {}
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError:
        return {}
    return payload if isinstance(payload, dict) else {}


def resolve_session_key(payload: dict) -> str:
    branch = git_branch()
    conversation_id = str(payload.get("conversation_id", "")).strip()
    if branch and conversation_id:
        return f"{branch}:{conversation_id[:8]}"
    return branch or conversation_id


def recent_context_summary(session_key: str, *, limit: int = 3) -> str:
    if not session_key:
        return ""

    try:
        branch = git_branch()
        rows = list_context(limit=limit, session_key=session_key)
        if not rows and branch and branch != session_key:
            rows = list_context(limit=limit, session_key=branch)
    except Exception:
        return ""

    if not rows:
        return ""

    blocks = [
        format_entry(*row, content_limit=800)
        for row in rows
    ]
    return "## Recent context memory\n\n" + "\n\n".join(blocks)


def session_start() -> int:
    payload = read_hook_input()
    session_key = resolve_session_key(payload)
    context = MEMORY_FIRST_CONTEXT.format(session_key=session_key or "(git branch)")
    recent = recent_context_summary(session_key)
    if recent:
        context = f"{context}\n\n{recent}"

    print(json.dumps({"additional_context": context}))
    return 0


def pre_compact() -> int:
    import os

    payload = read_hook_input()
    transcript_path = (
        str(payload.get("transcript_path", "")).strip()
        or os.environ.get("CURSOR_TRANSCRIPT_PATH", "").strip()
    )

    trigger = str(payload.get("trigger", "auto"))
    conversation_id = str(payload.get("conversation_id", "")).strip()
    session_key = resolve_session_key(payload)

    user_message = ""
    if not transcript_path:
        user_message = "Context compaction started (no transcript path; agent save required)."
        print(json.dumps({"user_message": user_message}))
        return 0

    try:
        entry_id = snapshot_transcript(
            transcript_path,
            session_key=session_key,
            title=f"Pre-compaction snapshot ({trigger})",
            tags=["pre-compact", "auto"] + ([conversation_id[:8]] if conversation_id else []),
        )
        user_message = (
            f"Saved pre-compaction context snapshot (entry id={entry_id}) "
            f"before {trigger} compaction."
        )
    except Exception as error:
        user_message = (
            "Context compaction started; automatic transcript snapshot failed "
            f"({error}). Save context manually with save_context_memory."
        )

    print(json.dumps({"user_message": user_message}))
    return 0


def main() -> int:
    if len(sys.argv) != 2:
        print(
            "Usage: .venv/bin/python tools/memory_hooks.py "
            "<session-start|pre-compact>",
            file=sys.stderr,
        )
        return 1

    command = sys.argv[1]
    if command == "session-start":
        return session_start()
    if command == "pre-compact":
        return pre_compact()

    print(f"Unknown hook command: {command}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
