#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

legacy_part="gpt-4o"
mini_suffix="-mini"
legacy_token="${legacy_part}${mini_suffix}"
legacy_pattern="${legacy_token}|openai/${legacy_token}"

mapfile -t matches < <(git grep -nE "${legacy_pattern}" -- . || true)

if [[ "${#matches[@]}" -gt 0 ]]; then
  echo "legacy model reference policy failed: found ${#matches[@]} match(es)" >&2
  printf '%s\n' "${matches[@]}" >&2
  exit 1
fi

echo "legacy model reference policy passed"
