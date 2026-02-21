#!/usr/bin/env bash
set -euo pipefail

scan_root="${1:-crates}"

if ! command -v rg >/dev/null 2>&1; then
  echo "error: ripgrep (rg) is required" >&2
  exit 1
fi

if [ ! -d "${scan_root}" ]; then
  echo "error: scan root does not exist: ${scan_root}" >&2
  exit 1
fi

is_test_path() {
  local path="$1"
  [[ "${path}" == *"/tests/"* ]] || [[ "${path}" == *"/src/tests/"* ]] || [[ "${path}" == *"tests.rs" ]] || [[ "${path}" == *"_test.rs" ]]
}

print_group() {
  local title="$1"
  shift
  echo "${title}:"
  if [ "$#" -eq 0 ]; then
    echo "  (none)"
    return
  fi
  for entry in "$@"; do
    echo "  ${entry}"
  done
}

summarize_matches() {
  local label="$1"
  local matches="$2"
  local cleaned
  cleaned="$(printf '%s\n' "${matches}" | sed '/^$/d' | sort || true)"

  local total=0
  local test_count=0
  local non_test_count=0
  local test_lines=()
  local non_test_lines=()

  while IFS= read -r line; do
    [ -z "${line}" ] && continue
    total=$((total + 1))
    local path="${line%%:*}"
    if is_test_path "${path}"; then
      test_count=$((test_count + 1))
      test_lines+=("${line}")
    else
      non_test_count=$((non_test_count + 1))
      non_test_lines+=("${line}")
    fi
  done <<< "${cleaned}"

  echo "${label}_total=${total}"
  echo "${label}_test_path=${test_count}"
  echo "${label}_non_test_path=${non_test_count}"
  print_group "${label}_test_matches" "${test_lines[@]}"
  print_group "${label}_non_test_matches" "${non_test_lines[@]}"
}

panic_matches="$(rg -n --no-heading --glob '*.rs' 'panic!\(' "${scan_root}" || true)"
unsafe_matches="$(rg -n --no-heading --glob '*.rs' '\bunsafe\s*\{|\bunsafe\s+fn\b' "${scan_root}" || true)"

echo "panic_unsafe_audit_root=${scan_root}"
summarize_matches "panic" "${panic_matches}"
summarize_matches "unsafe" "${unsafe_matches}"
