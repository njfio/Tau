#!/usr/bin/env bash
set -euo pipefail

if [[ "${TAU_OPENAI_EXPERIMENTAL_DIRECT_LIVE:-}" != "1" && "${TAU_OPENAI_EXPERIMENTAL_DIRECT_LIVE:-}" != "true" ]]; then
  printf '%s\n' 'skip: set TAU_OPENAI_EXPERIMENTAL_DIRECT_LIVE=1 to opt into the live openai_experimental_direct transport probe'
  cargo test -p tau-provider openai_experimental_direct_transport -- --test-threads=1
  exit 0
fi

if [[ -z "${TAU_OPENAI_EXPERIMENTAL_DIRECT_BEARER:-}" ]]; then
  printf '%s\n' 'skip: TAU_OPENAI_EXPERIMENTAL_DIRECT_BEARER is required for the live openai_experimental_direct transport probe'
  cargo test -p tau-provider openai_experimental_direct_transport -- --test-threads=1
  exit 0
fi

cargo test -p tau-provider openai_experimental_direct_transport_live_or_skip_safe_responses_probe -- --test-threads=1 --nocapture