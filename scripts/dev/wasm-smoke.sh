#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

if ! command -v rustup >/dev/null 2>&1; then
  echo "error: rustup is required for wasm-smoke.sh" >&2
  exit 1
fi

rustup target add wasm32-unknown-unknown >/dev/null
export RUSTC="$(rustup which --toolchain stable rustc)"

check_wasm_target() {
  local package="$1"
  echo "==> wasm check: ${package}"
  rustup run stable cargo check -p "${package}" --target wasm32-unknown-unknown
}

check_wasm_target "kamn-core"
check_wasm_target "kamn-sdk"
check_wasm_target "tau-access"
check_wasm_target "tau-deployment"

"${REPO_ROOT}/scripts/edge/cloudflare-wasm-smoke.sh"
"${REPO_ROOT}/scripts/edge/deno-wasm-smoke.sh"

echo "wasm smoke summary: status=pass packages=4 edge_harnesses=2"
