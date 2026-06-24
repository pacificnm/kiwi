#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

profile=debug
args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --release)
      profile=release
      shift
      ;;
    --)
      shift
      args+=("$@")
      break
      ;;
    *)
      args+=("$1")
      shift
      ;;
  esac
done

binary="${ROOT}/target/${profile}/kiwi"

if [[ ! -x "$binary" ]]; then
  echo "ERROR: kiwi binary not found at ${binary}" >&2
  if [[ "$profile" == release ]]; then
    echo "Build it first: ./scripts/build.sh --release" >&2
  else
    echo "Build it first: ./scripts/build.sh" >&2
  fi
  exit 1
fi

exec "$binary" "${args[@]}"
