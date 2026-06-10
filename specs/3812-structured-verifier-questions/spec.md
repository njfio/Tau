# Spec 3812: Structured Repo-Aware Verifier Questions

Status: Implemented

## Problem Statement

Tau now records actionable repo-aware verifier missing inputs, but queue workers
and operators still need to parse `missing_inputs` prose to ask the right
follow-up. The same package/filter blockers should also appear as structured
clarifying questions with stable reason codes.

## Scope

In:

- Preserve existing `missing_inputs` and fail-closed blocked behavior.
- Add structured clarifying questions for missing Cargo package and missing safe
  test-filter inputs.
- Persist the questions in the existing intake record.

Out:

- New persisted schema fields.
- Provider-based verifier invention.
- Changing ready-intake behavior when verifier authority is already supplied.

## Acceptance Criteria

AC-1: Given a repo-aware Rust issue that names a real package but lacks an exact
safe test filter, when Tau blocks intake, then `clarifying_questions` contains
reason code `repo_aware_test_filter` with the exact safe-filter required input.

AC-2: Given a repo-aware Rust issue that names a package absent from Cargo
metadata, when Tau blocks intake, then `clarifying_questions` contains reason
code `repo_aware_cargo_package` with the real-package required input and
available package context.

AC-3: The structured questions persist and reload through `issue_intake_status`.

## Conformance Cases

| Case | Maps | Tier | Input | Expected |
| --- | --- | --- | --- | --- |
| C-01 | AC-1/AC-3 | Conformance | Fixture repo with package `fixture-cli`; issue names package but no filter | Blocked intake persists `repo_aware_test_filter` question |
| C-02 | AC-2/AC-3 | Conformance | Fixture repo with package `fixture-cli`; issue names `missing-crate` and exact filter | Blocked intake persists `repo_aware_cargo_package` question |

## Success Signals

- `CARGO_TARGET_DIR=/tmp/rust_pi-3812-target cargo test -p tau-runtime spec_3812 -- --test-threads=1` passes.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3812-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` passes.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3812-target cargo clippy -p tau-runtime --lib --tests -- -D warnings` passes.
- `cargo fmt --check`, oversized-file guard, roadmap sync check, and `git diff --check` pass.
