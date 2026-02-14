#!/usr/bin/env bash
set -euo pipefail

if ! command -v deno >/dev/null 2>&1; then
  echo "deno wasm smoke: skipped (deno not installed)"
  exit 0
fi

echo "deno wasm smoke: $(deno --version | head -n 1)"
