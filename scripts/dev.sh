#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
COINZ_ORIGIN=http://localhost:5173 cargo run -- serve &
backend_pid=$!
pnpm --dir frontend dev &
frontend_pid=$!
trap 'kill "$backend_pid" "$frontend_pid" 2>/dev/null || true; wait || true' EXIT INT TERM
wait -n "$backend_pid" "$frontend_pid"
