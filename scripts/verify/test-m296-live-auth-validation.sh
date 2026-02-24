#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFY_SCRIPT="${SCRIPT_DIR}/m296-live-auth-validation.sh"

assert_equals() {
  local expected="$1"
  local actual="$2"
  local label="$3"
  if [[ "${expected}" != "${actual}" ]]; then
    echo "assertion failed (${label}): expected '${expected}' got '${actual}'" >&2
    exit 1
  fi
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected output to contain '${needle}'" >&2
    echo "actual output:" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

clean_env=(
  OPENAI_API_KEY=
  TAU_API_KEY=
  OPENROUTER_API_KEY=
  TAU_OPENROUTER_API_KEY=
  DEEPSEEK_API_KEY=
  TAU_DEEPSEEK_API_KEY=
  XAI_API_KEY=
  MISTRAL_API_KEY=
  GROQ_API_KEY=
  ANTHROPIC_API_KEY=
  GEMINI_API_KEY=
  GOOGLE_API_KEY=
)

# Regression: disabled mode should skip with code 20.
set +e
disabled_output="$(
  env "${clean_env[@]}" \
  TAU_M296_AUTH_LIVE_ENABLE="0" \
  "${VERIFY_SCRIPT}" 2>&1
)"
disabled_rc=$?
set -e
assert_equals "20" "${disabled_rc}" "disabled skip exit code"
assert_contains "${disabled_output}" "skip: live auth validation disabled" "disabled skip output"

# Regression: enabled mode with missing key file should skip with code 20.
set +e
missing_key_output="$(
  env "${clean_env[@]}" \
  TAU_M296_AUTH_LIVE_ENABLE="1" \
  TAU_PROVIDER_KEYS_FILE="${tmp_dir}/missing.env" \
  "${VERIFY_SCRIPT}" 2>&1
)"
missing_key_rc=$?
set -e
assert_equals "20" "${missing_key_rc}" "missing key file skip exit code"
assert_contains "${missing_key_output}" "skip: provider key file not found" "missing key file skip output"

# Regression: enabled mode with key file lacking usable keys should skip with code 20.
empty_key_file="${tmp_dir}/empty.env"
cat > "${empty_key_file}" <<'EOF'
PLACEHOLDER_KEY=
EOF
set +e
no_live_key_output="$(
  env "${clean_env[@]}" \
  TAU_M296_AUTH_LIVE_ENABLE="1" \
  TAU_PROVIDER_KEYS_FILE="${empty_key_file}" \
  "${VERIFY_SCRIPT}" 2>&1
)"
no_live_key_rc=$?
set -e
assert_equals "20" "${no_live_key_rc}" "no live key skip exit code"
assert_contains "${no_live_key_output}" "skip: provider key file contains no usable live auth keys" "no live key skip output"

# Functional: mock mode should pass deterministically without live network calls.
mock_key_file="${tmp_dir}/mock.env"
cat > "${mock_key_file}" <<'EOF'
OPENAI_API_KEY=mock-live-key
EOF
set +e
mock_output="$(
  env "${clean_env[@]}" \
  TAU_M296_AUTH_LIVE_ENABLE="0" \
  TAU_PROVIDER_KEYS_FILE="${mock_key_file}" \
  TAU_M296_AUTH_LIVE_MOCK_MODE="1" \
  "${VERIFY_SCRIPT}" 2>&1
)"
mock_rc=$?
set -e
assert_equals "0" "${mock_rc}" "mock mode pass exit code"
assert_contains "${mock_output}" "m296 live auth validation passed" "mock mode pass output"

# Regression: mock mode fail flag should fail closed.
set +e
mock_fail_output="$(
  env "${clean_env[@]}" \
  TAU_M296_AUTH_LIVE_ENABLE="0" \
  TAU_PROVIDER_KEYS_FILE="${mock_key_file}" \
  TAU_M296_AUTH_LIVE_MOCK_MODE="1" \
  TAU_M296_AUTH_LIVE_MOCK_FAIL="1" \
  "${VERIFY_SCRIPT}" 2>&1
)"
mock_fail_rc=$?
set -e
assert_equals "1" "${mock_fail_rc}" "mock mode fail exit code"
assert_contains "${mock_fail_output}" "m296 live auth validation failed" "mock mode fail output"

echo "m296-live-auth-validation tests passed"
