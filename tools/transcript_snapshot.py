"""Extract conversation text from Cursor transcript JSONL and save to context memory."""

from __future__ import annotations

import json
import sys
from pathlib import Path

from context_memory import save_context
from memory_common import PROJECT_ROOT


MAX_TRANSCRIPT_CHARS = 30_000


def git_branch() -> str:
    import subprocess

    try:
        result = subprocess.run(
            ["git", "branch", "--show-current"],
            capture_output=True,
            text=True,
            timeout=5,
            cwd=PROJECT_ROOT,
            check=False,
        )
        return result.stdout.strip()
    except (OSError, subprocess.TimeoutExpired):
        return ""


def extract_transcript_text(path: Path, *, max_chars: int = MAX_TRANSCRIPT_CHARS) -> str:
    if not path.is_file():
        raise FileNotFoundError(f"transcript not found: {path}")

    sections: list[str] = []
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line:
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue

        role = str(entry.get("role", "unknown"))
        message = entry.get("message")
        if not isinstance(message, dict):
            continue

        content = message.get("content")
        if not isinstance(content, list):
            continue

        parts: list[str] = []
        for block in content:
            if not isinstance(block, dict):
                continue
            block_type = block.get("type")
            if block_type == "text":
                text = str(block.get("text", "")).strip()
                if text:
                    parts.append(text)
            elif block_type == "tool_use":
                tool_name = str(block.get("name", "tool"))
                parts.append(f"[tool: {tool_name}]")

        if parts:
            sections.append(f"## {role}\n" + "\n".join(parts))

    full_text = "\n\n".join(sections).strip()
    if not full_text:
        raise ValueError("transcript contained no extractable text")

    if len(full_text) <= max_chars:
        return full_text

    marker = "...[earlier messages truncated]...\n\n"
    tail_budget = max_chars - len(marker)
    return marker + full_text[-tail_budget:]


def snapshot_transcript(
    transcript_path: str | Path,
    *,
    session_key: str = "",
    title: str = "Pre-compaction snapshot",
    tags: list[str] | None = None,
) -> int:
    text = extract_transcript_text(Path(transcript_path))
    tag_list = list(tags or [])
    for tag in ("pre-compact", "auto"):
        if tag not in tag_list:
            tag_list.append(tag)

    return save_context(
        text,
        title=title,
        session_key=session_key,
        tags=tag_list,
    )


def main() -> int:
    if len(sys.argv) < 2:
        print(
            "Usage: .venv/bin/python tools/transcript_snapshot.py <transcript_path> "
            "[session_key]",
            file=sys.stderr,
        )
        return 1

    transcript_path = sys.argv[1]
    session_key = sys.argv[2] if len(sys.argv) > 2 else git_branch()

    try:
        entry_id = snapshot_transcript(transcript_path, session_key=session_key)
    except Exception as error:
        print(f"ERROR: transcript snapshot failed: {error}", file=sys.stderr)
        return 1

    print(entry_id)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
