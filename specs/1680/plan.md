# Issue 1680 Plan

Status: Reviewed

## Approach

1. Tests-first:
   - run `scripts/dev/test-github-issues-runtime-domain-split.sh` before it
     exists to capture RED.
2. Add split-conformance harness:
   - create `scripts/dev/test-github-issues-runtime-domain-split.sh` to verify:
     - `github_issues_runtime.rs` line count `< 4000`
     - required module declarations exist in `github_issues_runtime.rs`
     - extracted domain files exist under `src/github_issues_runtime/`
3. Validate behavior/quality:
   - run harness for GREEN
   - run targeted tests and quality checks:
     - `cargo test -p tau-github-issues-runtime github_issues_runtime`
     - `scripts/dev/roadmap-status-sync.sh --check --quiet`
     - `cargo fmt --check`
     - `cargo clippy -p tau-github-issues-runtime -- -D warnings`

## Affected Areas

- `scripts/dev/test-github-issues-runtime-domain-split.sh`
- `specs/1680/spec.md`
- `specs/1680/plan.md`
- `specs/1680/tasks.md`

## Risks And Mitigations

- Risk: harness markers drift if module names change.
  - Mitigation: keep marker list aligned with `mod` declarations in runtime file.
- Risk: false confidence from structure-only checks.
  - Mitigation: run targeted runtime tests in the same issue verification set.

## Interfaces / Contracts

- No runtime behavior/interface changes.
- Existing module boundaries under
  `crates/tau-github-issues-runtime/src/github_issues_runtime/` are asserted.
- Harness output is the structural conformance contract artifact for this issue.

## ADR

No dependency/protocol/architecture decision changes; ADR not required.
