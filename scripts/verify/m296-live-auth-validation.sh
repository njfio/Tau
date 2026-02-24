#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KEY_FILE="${TAU_PROVIDER_KEYS_FILE:-${ROOT_DIR}/.tau/provider-keys.env}"
SKIP_EXIT_CODE=20
MOCK_MODE="${TAU_M296_AUTH_LIVE_MOCK_MODE:-0}"
MOCK_FAIL="${TAU_M296_AUTH_LIVE_MOCK_FAIL:-0}"

if [[ "${MOCK_MODE}" == "1" ]]; then
  echo "==> deterministic auth conformance selectors (mock mode)"
  if [[ "${MOCK_FAIL}" == "1" ]]; then
    echo "m296 live auth validation failed (mock mode)"
    exit 1
  fi
  echo "==> live provider auth smoke (mock mode)"
  echo "m296 live auth validation passed"
  exit 0
fi

if [[ "${TAU_M296_AUTH_LIVE_ENABLE:-0}" != "1" ]]; then
  echo "skip: live auth validation disabled (set TAU_M296_AUTH_LIVE_ENABLE=1)"
  exit "${SKIP_EXIT_CODE}"
fi

if [[ ! -f "${KEY_FILE}" ]]; then
  echo "skip: provider key file not found: ${KEY_FILE}"
  exit "${SKIP_EXIT_CODE}"
fi

set -a
# shellcheck disable=SC1090
source "${KEY_FILE}"
set +a

has_live_key="0"
for key in \
  OPENAI_API_KEY TAU_API_KEY OPENROUTER_API_KEY TAU_OPENROUTER_API_KEY DEEPSEEK_API_KEY \
  TAU_DEEPSEEK_API_KEY XAI_API_KEY MISTRAL_API_KEY GROQ_API_KEY ANTHROPIC_API_KEY \
  GEMINI_API_KEY GOOGLE_API_KEY
do
  if [[ -n "${!key:-}" ]]; then
    has_live_key="1"
    break
  fi
done

if [[ "${has_live_key}" != "1" ]]; then
  echo "skip: provider key file contains no usable live auth keys"
  exit "${SKIP_EXIT_CODE}"
fi

echo "==> deterministic auth conformance selectors"
(cd "${ROOT_DIR}" && cargo test -p tau-provider --test auth_workflow_conformance -- --nocapture)
(cd "${ROOT_DIR}" && cargo test -p tau-gateway conformance_gateway_auth_session_endpoint_enforces_mode_and_password_lifecycle_contracts -- --nocapture)

echo "==> live provider auth smoke"
(
  cd "${ROOT_DIR}" && \
  TAU_PROVIDER_KEYS_FILE="${KEY_FILE}" \
  bash scripts/dev/provider-live-smoke.sh
)

echo "m296 live auth validation passed"
