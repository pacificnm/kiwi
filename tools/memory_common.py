"""Shared helpers for Kiwi project-memory tools."""

from __future__ import annotations

import os
from collections.abc import Sequence
from pathlib import Path


DEFAULT_DATABASE_URL = "postgresql:///kiwi_memory?host=/var/run/postgresql"
PROJECT_ROOT = Path(__file__).resolve().parents[1]

# Env-var name → TOML path inside [agent.providers.<name>]
_PROVIDER_KEY_VARS = {
    "OPENAI_API_KEY":    "openai",
    "ANTHROPIC_API_KEY": "claude",
    "CURSOR_API_KEY":    "cursor",
}


def load_project_env() -> None:
    """Load repo-local .env values without requiring python-dotenv."""
    env_path = PROJECT_ROOT / ".env"
    if not env_path.is_file():
        return

    for raw_line in env_path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue

        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip().strip('"').strip("'")

        if key and value and key not in os.environ:
            os.environ[key] = value


def load_kiwi_config_keys() -> None:
    """Read API keys from ~/.config/kiwi/config.toml and set them as env vars.

    The kiwi config is the source of truth when keys are managed via the
    Settings panel. Keys found here override values from .env so that
    rotating a key in the UI automatically updates the indexing scripts.
    """
    config_path = Path.home() / ".config" / "kiwi" / "config.toml"
    if not config_path.is_file():
        return

    try:
        try:
            import tomllib  # Python 3.11+ stdlib
            data = tomllib.loads(config_path.read_text(encoding="utf-8"))
        except ImportError:
            import tomli  # type: ignore[import]
            data = tomli.loads(config_path.read_text(encoding="utf-8"))
    except Exception:
        return

    providers = data.get("agent", {}).get("providers", {})
    for env_var, provider_name in _PROVIDER_KEY_VARS.items():
        key = providers.get(provider_name, {}).get("api_key", "").strip()
        if key:
            os.environ[env_var] = key


def database_url() -> str:
    load_project_env()
    load_kiwi_config_keys()
    return os.environ.get("DATABASE_URL", DEFAULT_DATABASE_URL)


def vector_literal(values: Sequence[float]) -> str:
    """Return a pgvector text literal without requiring the pgvector Python package."""
    return "[" + ",".join(str(float(value)) for value in values) + "]"


load_project_env()
load_kiwi_config_keys()
