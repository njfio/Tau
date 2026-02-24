#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTPUT_DIR="${TAU_M301_OUTPUT_DIR:-${ROOT_DIR}/artifacts/rl-e2e}"
RUN_ID="${TAU_M301_RUN_ID:-m301-gate}"
MOCK_MODE="${TAU_M301_MOCK_MODE:-0}"
VERIFY_ONLY="${TAU_M301_VERIFY_ONLY:-0}"
CARGO_TARGET_DIR_VALUE="${TAU_M301_CARGO_TARGET_DIR:-target-fast}"

require_cmd() {
  local name="$1"
  if ! command -v "${name}" >/dev/null 2>&1; then
    echo "error: required command '${name}' not found" >&2
    exit 1
  fi
}

normalize_run_id() {
  local raw="$1"
  local normalized
  normalized="$(printf '%s' "${raw}" | tr -c 'A-Za-z0-9_-' '-')"
  if [[ -z "${normalized}" ]]; then
    normalized="deterministic"
  fi
  printf '%s' "${normalized}"
}

require_cmd jq

normalized_run_id="$(normalize_run_id "${RUN_ID}")"
artifact_path="${OUTPUT_DIR}/rl-e2e-harness-v1-${normalized_run_id}.json"

if [[ "${VERIFY_ONLY}" != "1" ]]; then
  if [[ "${MOCK_MODE}" == "1" ]]; then
    mkdir -p "${OUTPUT_DIR}"
    cat > "${artifact_path}" <<JSON
{
  "schema_version": 1,
  "run_id": "${RUN_ID}",
  "rollout_summary": {
    "total_rollouts": 6,
    "succeeded": 6,
    "failed": 0,
    "cancelled": 0
  },
  "gae_summary": {
    "advantages_len": 6,
    "mean_advantage": 0.12,
    "mean_return": 0.98,
    "normalized": true
  },
  "ppo_summary": {
    "mini_batch_count": 3,
    "optimizer_step_count": 6,
    "mean_total_loss": 0.05,
    "observed_approx_kl": 0.01,
    "early_stop_triggered": false
  },
  "promotion_gate": {
    "policy_improvement_significant": true,
    "promotion_allowed": true,
    "safety_regression": -0.01,
    "max_safety_regression": 0.05,
    "reason_codes": []
  },
  "rollback_gate": {
    "rollback_required": false,
    "failing_checks": [],
    "reason_codes": []
  },
  "checks": [
    {"id":"rollout_completion","passed":true,"detail":"ok"},
    {"id":"policy_improvement_significance","passed":true,"detail":"ok"},
    {"id":"checkpoint_promotion_gate","passed":true,"detail":"ok"},
    {"id":"rollback_gate","passed":true,"detail":"ok"}
  ],
  "pass": true
}
JSON
  else
    (
      cd "${ROOT_DIR}" && \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR_VALUE}" \
      cargo run -p tau-trainer --bin rl_e2e_harness -- \
        --run-id "${RUN_ID}" \
        --output-dir "${OUTPUT_DIR}"
    )
  fi
fi

if [[ ! -f "${artifact_path}" ]]; then
  echo "error: missing RL artifact: ${artifact_path}" >&2
  exit 1
fi

jq -e '.promotion_gate | type == "object"' "${artifact_path}" >/dev/null
jq -e '.promotion_gate.promotion_allowed | type == "boolean"' "${artifact_path}" >/dev/null
jq -e '.promotion_gate.policy_improvement_significant | type == "boolean"' "${artifact_path}" >/dev/null
jq -e '.promotion_gate.reason_codes | type == "array"' "${artifact_path}" >/dev/null
jq -e '.rollback_gate | type == "object"' "${artifact_path}" >/dev/null
jq -e '.rollback_gate.rollback_required | type == "boolean"' "${artifact_path}" >/dev/null
jq -e '.rollback_gate.failing_checks | type == "array"' "${artifact_path}" >/dev/null
jq -e '.rollback_gate.reason_codes | type == "array"' "${artifact_path}" >/dev/null
jq -e '(.checks | map(select(.id == "checkpoint_promotion_gate")) | length) == 1' "${artifact_path}" >/dev/null
jq -e '(.checks | map(select(.id == "rollback_gate")) | length) == 1' "${artifact_path}" >/dev/null
jq -e 'if .promotion_gate.promotion_allowed then true else (.promotion_gate.reason_codes | length > 0) end' "${artifact_path}" >/dev/null
jq -e 'if .rollback_gate.rollback_required then (.rollback_gate.reason_codes | length > 0) else true end' "${artifact_path}" >/dev/null

echo "m301 rl gate verification passed: ${artifact_path}"
