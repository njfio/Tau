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

assert_equals() {
  local actual="$1"
  local expected="$2"
  local label="$3"
  if [[ "${actual}" != "${expected}" ]]; then
    echo "assertion failed (${label}): expected '${expected}', got '${actual}'" >&2
    exit 1
  fi
}

init_minimal_workspace_repo() {
  local repo_root="$1"
  mkdir -p \
    "${repo_root}/scripts/dev" \
    "${repo_root}/crates/app/src" \
    "${repo_root}/crates/other/src"
  cp "${FAST_VALIDATE}" "${repo_root}/scripts/dev/fast-validate.sh"
  chmod +x "${repo_root}/scripts/dev/fast-validate.sh"

  cat > "${repo_root}/Cargo.toml" <<'EOF'
[workspace]
members = ["crates/app", "crates/other"]
resolver = "2"
EOF

  cat > "${repo_root}/crates/app/Cargo.toml" <<'EOF'
[package]
name = "app"
version = "0.1.0"
edition = "2021"
EOF

  cat > "${repo_root}/crates/other/Cargo.toml" <<'EOF'
[package]
name = "other"
version = "0.1.0"
edition = "2021"
EOF
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

fmt_scope_source="${tmp_root}/fmt-scope-source"
fmt_scope_remote="${tmp_root}/fmt-scope-remote.git"
fmt_scope_shallow="${tmp_root}/fmt-scope-shallow"

init_minimal_workspace_repo "${fmt_scope_source}"
cat > "${fmt_scope_source}/crates/app/src/lib.rs" <<'EOF'
pub fn app() -> u32 {
    1
}
EOF
cat > "${fmt_scope_source}/crates/other/src/lib.rs" <<'EOF'
pub fn other()->u32{1}
EOF

(
  cd "${fmt_scope_source}"
  git init -q
  git config user.name test
  git config user.email test@example.com
  git add .
  git commit -q -m "base"
  base_sha="$(git rev-parse HEAD)"
  cat > crates/app/src/lib.rs <<'EOF'
pub fn app() -> u32 {
    2
}
EOF
  git add crates/app/src/lib.rs
  git commit -q -m "head"
  printf '%s\n' "${base_sha}" > "${tmp_root}/fmt_scope_base_sha"
)

git clone --bare -q "${fmt_scope_source}" "${fmt_scope_remote}"
git clone --depth 1 -q "file://${fmt_scope_remote}" "${fmt_scope_shallow}"

fmt_scope_base_sha="$(cat "${tmp_root}/fmt_scope_base_sha")"
(
  cd "${fmt_scope_shallow}"
  git fetch --depth=1 origin "${fmt_scope_base_sha}" >/dev/null 2>&1
  if output="$(./scripts/dev/fast-validate.sh --check-only --base "${fmt_scope_base_sha}" 2>&1)"; then
    status=0
  else
    status=$?
  fi
  assert_equals "${status}" "0" "unrelated fmt drift outside diff should not fail scoped validation"
  assert_contains "${output}" "changed_rust_files=1" "scoped fmt run should report changed Rust files"
  assert_not_contains "${output}" "crates/other/src/lib.rs" "scoped fmt run should ignore unrelated unformatted file"
)

fmt_changed_source="${tmp_root}/fmt-changed-source"
fmt_changed_remote="${tmp_root}/fmt-changed-remote.git"
fmt_changed_shallow="${tmp_root}/fmt-changed-shallow"

init_minimal_workspace_repo "${fmt_changed_source}"
cat > "${fmt_changed_source}/crates/app/src/lib.rs" <<'EOF'
pub fn app() -> u32 {
    1
}
EOF
cat > "${fmt_changed_source}/crates/other/src/lib.rs" <<'EOF'
pub fn other() -> u32 {
    1
}
EOF

(
  cd "${fmt_changed_source}"
  git init -q
  git config user.name test
  git config user.email test@example.com
  git add .
  git commit -q -m "base"
  base_sha="$(git rev-parse HEAD)"
  cat > crates/app/src/lib.rs <<'EOF'
pub fn app()->u32{2}
EOF
  git add crates/app/src/lib.rs
  git commit -q -m "head"
  printf '%s\n' "${base_sha}" > "${tmp_root}/fmt_changed_base_sha"
)

git clone --bare -q "${fmt_changed_source}" "${fmt_changed_remote}"
git clone --depth 1 -q "file://${fmt_changed_remote}" "${fmt_changed_shallow}"

fmt_changed_base_sha="$(cat "${tmp_root}/fmt_changed_base_sha")"
(
  cd "${fmt_changed_shallow}"
  git fetch --depth=1 origin "${fmt_changed_base_sha}" >/dev/null 2>&1
  if output="$(./scripts/dev/fast-validate.sh --check-only --base "${fmt_changed_base_sha}" 2>&1)"; then
    status=0
  else
    status=$?
  fi
  assert_equals "${status}" "1" "changed unformatted Rust file should still fail scoped validation"
  assert_contains "${output}" "crates/app/src/lib.rs" "fmt failure should point at changed Rust file"
  assert_not_contains "${output}" "crates/other/src/lib.rs" "fmt failure should stay scoped to changed Rust file"
)

help_output="$("${FAST_VALIDATE}" --help)"
assert_contains "${help_output}" "--skip-fmt" "help output should document skip-fmt option"

echo "fast-validate scope tests passed"
