#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PYTHON="${ROOT}/.venv/bin/python"

if [[ ! -x "$PYTHON" ]]; then
  echo "ERROR: virtual environment not found at ${ROOT}/.venv" >&2
  echo "Create it with: python3 -m venv .venv && .venv/bin/pip install -r tools/requirements.txt" >&2
  exit 1
fi

exec "$PYTHON" "${ROOT}/tools/index_memory.py" "$@"
