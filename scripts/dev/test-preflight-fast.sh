#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFLIGHT="${SCRIPT_DIR}/preflight-fast.sh"

assert_equals() {
  local expected="$1"
  local actual="$2"
  local label="$3"
  if [[ "${expected}" != "${actual}" ]]; then
    echo "assertion failed (${label}): expected '${expected}' got '${actual}'" >&2
    exit 1
  fi
}

assert_file_contains() {
  local path="$1"
  local needle="$2"
  local label="$3"
  local content
  content="$(cat "${path}")"
  if [[ "${content}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected '${needle}' in ${path}" >&2
    echo "actual content: ${content}" >&2
    exit 1
  fi
}

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

roadmap_pass="${tmp_dir}/roadmap-pass.sh"
roadmap_fail="${tmp_dir}/roadmap-fail.sh"
fast_validate="${tmp_dir}/fast-validate.sh"

cat >"${roadmap_pass}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" > "${TEST_ROADMAP_ARGS_FILE}"
exit 0
EOF

cat >"${roadmap_fail}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" > "${TEST_ROADMAP_ARGS_FILE}"
exit 7
EOF

cat >"${fast_validate}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" > "${TEST_FAST_ARGS_FILE}"
touch "${TEST_FAST_CALLED_FILE}"
exit 0
EOF

chmod +x "${roadmap_pass}" "${roadmap_fail}" "${fast_validate}"

roadmap_args_file="${tmp_dir}/roadmap-args.txt"
fast_args_file="${tmp_dir}/fast-args.txt"
fast_called_file="${tmp_dir}/fast-called.txt"
export TEST_ROADMAP_ARGS_FILE="${roadmap_args_file}"
export TEST_FAST_ARGS_FILE="${fast_args_file}"
export TEST_FAST_CALLED_FILE="${fast_called_file}"

# Functional: success path runs roadmap check first and forwards args.
TAU_ROADMAP_SYNC_BIN="${roadmap_pass}" \
TAU_FAST_VALIDATE_BIN="${fast_validate}" \
"${PREFLIGHT}" --check-only --base origin/master

assert_file_contains "${roadmap_args_file}" "--check --quiet" "roadmap args"
assert_file_contains "${fast_args_file}" "--check-only --base origin/master" "passthrough args"
if [[ ! -f "${fast_called_file}" ]]; then
  echo "assertion failed (success path): fast-validate should be called" >&2
  exit 1
fi

# Regression: roadmap failure fails closed and does not call fast-validate.
rm -f "${fast_called_file}"
set +e
TAU_ROADMAP_SYNC_BIN="${roadmap_fail}" \
TAU_FAST_VALIDATE_BIN="${fast_validate}" \
"${PREFLIGHT}" --check-only >/dev/null 2>&1
status=$?
set -e
assert_equals "7" "${status}" "roadmap failure exit code"
if [[ -f "${fast_called_file}" ]]; then
  echo "assertion failed (fail-closed): fast-validate must not run on roadmap failure" >&2
  exit 1
fi

echo "preflight-fast tests passed"
