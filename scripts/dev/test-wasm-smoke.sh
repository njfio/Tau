#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

bash -n scripts/dev/wasm-smoke.sh
bash -n scripts/edge/cloudflare-wasm-smoke.sh
bash -n scripts/edge/deno-wasm-smoke.sh

echo "wasm-smoke scripts syntax tests passed"
