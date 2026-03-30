#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${REPO_ROOT}"

FAST_VALIDATE="${SCRIPT_DIR}/fast-validate.sh"

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

assert_not_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" == *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected output to NOT contain '${needle}'" >&2
    echo "actual output:" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

output="$(printf 'crates/tau-cli/src/cli_args.rs\n' | "${FAST_VALIDATE}" --print-packages-from-stdin)"
assert_contains "${output}" "full_workspace=0" "crate file should not force workspace"
assert_contains "${output}" "package=tau-cli" "crate file should map to tau-cli package"
assert_contains "${output}" "package=tau-coding-agent" "tau-cli impact scope should include reverse dependents"

output="$(printf 'crates/tau-cli/src/cli_args.rs\n' | "${FAST_VALIDATE}" --print-packages-from-stdin --direct-packages-only)"
assert_contains "${output}" "full_workspace=0" "direct-only mode should not force workspace"
assert_contains "${output}" "package=tau-cli" "direct-only mode should include directly changed crate"
assert_not_contains "${output}" "package=tau-coding-agent" "direct-only mode should skip reverse dependents"

output="$(printf 'Cargo.toml\n' | "${FAST_VALIDATE}" --print-packages-from-stdin)"
assert_contains "${output}" "full_workspace=1" "workspace manifest should force full scope"

output="$(printf 'docs/README.md\n' | "${FAST_VALIDATE}" --print-packages-from-stdin)"
assert_contains "${output}" "full_workspace=0" "docs-only change should stay package-scoped"
assert_not_contains "${output}" "package=" "docs-only change should not emit package scope"

output="$(printf 'docs/README.md\n' | "${FAST_VALIDATE}" --print-packages-from-stdin --skip-fmt)"
assert_contains "${output}" "full_workspace=0" "skip-fmt should not affect package scope derivation"

output="$(printf 'crates/tau-cli/src/lib.rs\ncrates/tau-tools/src/lib.rs\n' | "${FAST_VALIDATE}" --print-packages-from-stdin)"
assert_contains "${output}" "package=tau-cli" "multi-crate input should include tau-cli"
assert_contains "${output}" "package=tau-tools" "multi-crate input should include tau-tools"
assert_contains "${output}" "package=tau-coding-agent" "tau-tools impact scope should include coding-agent"

tmp_root="$(mktemp -d)"
trap 'rm -rf "${tmp_root}"' EXIT

source_repo="${tmp_root}/source"
remote_repo="${tmp_root}/remote.git"
shallow_repo="${tmp_root}/shallow"

mkdir -p "${source_repo}/scripts/dev" "${source_repo}/docs"
cp "${FAST_VALIDATE}" "${source_repo}/scripts/dev/fast-validate.sh"
chmod +x "${source_repo}/scripts/dev/fast-validate.sh"

(
  cd "${source_repo}"
  git init -q
  git config user.name test
  git config user.email test@example.com
  echo "base" > docs/README.md
  git add .
  git commit -q -m "base"
  base_sha="$(git rev-parse HEAD)"
  echo "middle" >> docs/README.md
  git commit -qam "middle"
  echo "head" >> docs/README.md
  git commit -qam "head"
  printf '%s\n' "${base_sha}" > "${tmp_root}/base_sha"
)

git clone --bare -q "${source_repo}" "${remote_repo}"
git clone --depth 1 -q "file://${remote_repo}" "${shallow_repo}"

base_sha="$(cat "${tmp_root}/base_sha")"
(
  cd "${shallow_repo}"
  git fetch --depth=1 origin "${base_sha}" >/dev/null 2>&1
  output="$(./scripts/dev/fast-validate.sh --check-only --skip-fmt --base "${base_sha}" 2>&1)"
  assert_contains "${output}" "warning: base ref '${base_sha}' has no local merge base with HEAD; using two-dot diff fallback" "shallow-history case should emit bounded fallback warning"
  assert_not_contains "${output}" "fatal:" "shallow-history case should not leak raw git fatal output"
  assert_contains "${output}" "changed_files=1" "shallow-history case should preserve changed file scope"
  assert_contains "${output}" "no crate changes detected; fmt check completed" "shallow-history docs-only case should stay scoped"
)

help_output="$("${FAST_VALIDATE}" --help)"
assert_contains "${help_output}" "--skip-fmt" "help output should document skip-fmt option"

echo "fast-validate scope tests passed"
