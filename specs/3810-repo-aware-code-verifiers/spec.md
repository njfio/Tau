# Spec 3810: Repo-Aware Code Verifier Planning

Status: Reviewed

## Problem Statement

Tau can now derive a verifier for a narrow docs marker case, but bounded Rust or
CLI issues still require an operator to hand-write a verifier even when the
issue names a real crate and exact test filter. Tau should use repository
metadata to derive focused code verifiers only when the package and test filter
are concrete, real, and safe as argv tokens. If either part is missing, Tau must
continue to block before mutation.

## Scope

In:

- Use `cargo metadata --no-deps --format-version 1` to discover real package
  names for the target repo.
- Derive `cargo test -p <package> <test-filter>` only when the issue references
  a discovered package and a quoted/backticked safe test/filter token.
- Persist the derived verifier in the issue-intake verifier plan.
- Feed the derived verifier into `issue-to-merge` when edit/provider authority
  is present.

Out:

- General verifier invention for arbitrary issues.
- Shell-quoted multi-word filters.
- Guessing package names that are not present in `cargo metadata`.
- New CLI flags, provider changes, or live GitHub behavior.

## Acceptance Criteria

AC-1: Given a repo with package `fixture-cli`, an issue naming that package and
exact test filter `spec_3810_repo_aware_verifier`, and provider repair
authority, when `issue-to-merge` omits verifier commands, then Tau derives
`cargo test -p fixture-cli spec_3810_repo_aware_verifier` and reaches PR-ready.

AC-2: Given the successful path, when the outcome is inspected, then the
persisted intake plan has `decision=ready_to_run`, `plan_kind=rust`, and the
derived verifier command.

AC-3: Given a repo with package `fixture-cli` but no exact quoted/backticked
test filter, when `issue-to-merge` omits verifier commands, then Tau blocks
before job creation and records `verifier_authority_required`.

AC-4: Given an issue that references a package not present in `cargo metadata`,
Tau must not derive a command for that package.

## Conformance Cases

| Case | Maps | Tier | Input | Expected |
| --- | --- | --- | --- | --- |
| C-01 | AC-1/AC-2 | Conformance | Workspace fixture with package `fixture-cli`, issue token `spec_3810_repo_aware_verifier`, provider repair authority | Derived cargo test verifier runs and job reaches PR-ready |
| C-02 | AC-3 | Regression | Same repo and provider authority, but no exact test-filter token | Blocks before mutation with verifier authority required |
| C-03 | AC-4 | Regression | Same repo and exact test-filter token, but the issue references package `missing-crate` which is absent from `cargo metadata` | Blocks before mutation with verifier authority required |

## Success Signals

- `CARGO_TARGET_DIR=/tmp/rust_pi-3810-target cargo test -p tau-runtime spec_3810 -- --test-threads=1` passes.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3810-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` passes.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3810-target cargo clippy -p tau-runtime --lib --tests -- -D warnings` passes.
- `cargo fmt --check` and `git diff --check` pass.
