#!/usr/bin/env bash
set -euo pipefail

if ! command -v wrangler >/dev/null 2>&1; then
  echo "cloudflare wasm smoke: skipped (wrangler not installed)"
  exit 0
fi

version_output="$(wrangler --version)"
echo "cloudflare wasm smoke: wrangler=${version_output}"
