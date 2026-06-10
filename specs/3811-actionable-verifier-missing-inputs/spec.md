# Spec 3811: Actionable Verifier Missing Inputs

Status: Reviewed

## Problem Statement

Tau now derives focused verifiers for narrow docs and Rust/CLI issues, but when
repo-aware derivation fails the persisted verifier plan still falls back to
generic placeholders. Operators need to know the exact missing ingredient:
whether Tau needs a real Cargo package token, an exact safe test-filter token,
or normal mutation/provider authority.

## Scope

In:

- Preserve fail-closed behavior when verifier derivation cannot produce a
  concrete command.
- Add actionable repo-aware missing-input text for unresolved package names and
  missing/unsafe test filters.
- Keep the existing `missing_inputs` and `next_action` schema.

Out:

- Provider-based verifier invention.
- New CLI flags or schema fields.
- Guessing packages or test filters from broad issue text.

## Acceptance Criteria

AC-1: Given a Rust issue that names a real Cargo package but omits an exact safe
test-filter token, when Tau blocks before mutation, then the persisted verifier
plan names the missing exact quoted/backticked safe test filter.

AC-2: Given a Rust issue that names a package absent from `cargo metadata`, when
Tau blocks before mutation, then the persisted verifier plan names the missing
real Cargo package input and shows available package context.

AC-3: Given either blocked path, Tau must not create a job or run provider
repair.

## Conformance Cases

| Case | Maps | Tier | Input | Expected |
| --- | --- | --- | --- | --- |
| C-01 | AC-1/AC-3 | Conformance | Fixture repo with package `fixture-cli`; issue names package but no test filter | Blocks and `missing_inputs` names the exact quoted/backticked safe test filter |
| C-02 | AC-2/AC-3 | Conformance | Fixture repo with package `fixture-cli`; issue names `missing-crate` and exact test token | Blocks and `missing_inputs` names an actual Cargo package from metadata |

## Success Signals

- `CARGO_TARGET_DIR=/tmp/rust_pi-3811-target cargo test -p tau-runtime spec_3811 -- --test-threads=1` passes.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3811-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` passes.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3811-target cargo clippy -p tau-runtime --lib --tests -- -D warnings` passes.
- `cargo fmt --check` and `git diff --check` pass.
