# Spec 3809: Derived Docs Verifier for Issue-to-Merge

Status: Reviewed

## Problem Statement

`issue-to-merge` currently blocks whenever `--verifier-command` is omitted,
even when the issue is a bounded docs request and contains a concrete text
marker Tau can verify safely. That keeps the loop too manual for simple
documentation fixes. Tau should be able to derive a small docs verifier only
when the issue supplies an exact argv-safe marker, while preserving fail-closed
behavior for vague docs work and all broader coding work.

## Scope

In:

- Derive verifier commands for docs/readme issues that include a quoted or
  backticked single-token marker.
- Persist the intake record that explains the derived verifier plan.
- Run the normal durable issue-to-merge path using the derived verifier when
  provider/edit authority is present.
- Keep blocking when no concrete marker exists.

Out:

- General verifier invention for arbitrary coding tasks.
- Shell-quoted multi-word grep phrases.
- Provider-generated verifier plans.
- Live provider or live GitHub validation.

## Acceptance Criteria

AC-1: Given a docs issue with a concrete backticked marker and provider repair
authority, when `issue-to-merge` is called without `--verifier-command`, then Tau
derives `git diff --check` plus a concrete `grep` verifier and can reach
PR-ready through the normal durable job loop.

AC-2: Given the same successful path, when the outcome is inspected, then it
includes a persisted intake record with `decision=ready_to_run`, `plan_kind=docs`,
and the derived verifier commands.

AC-3: Given a docs issue without a quoted/backticked safe marker, when
`issue-to-merge` is called without `--verifier-command`, then Tau blocks before
creating a job and records `verifier_authority_required`.

AC-4: Given an unsafe, broad, or underspecified issue, the derived-docs verifier
path must not override the existing fail-closed intake classification.

## Conformance Cases

| Case | Maps | Tier | Input | Expected |
| --- | --- | --- | --- | --- |
| C-01 | AC-1/AC-2 | Conformance | Docs issue containing `tau_derived_docs_verifier`, provider repair authority, no verifier command | Job reaches PR-ready; intake records derived docs verifier |
| C-02 | AC-3 | Regression | Docs issue with no concrete marker, provider repair authority, no verifier command | Blocks before mutation with verifier authority required |
| C-03 | AC-4 | Regression | Existing broad/unsafe/underspecified intake tests | Existing classifications remain authoritative |

## Success Signals

- `CARGO_TARGET_DIR=/tmp/rust_pi-3809-target cargo test -p tau-runtime spec_3809 -- --test-threads=1` passes.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3809-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` passes.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3809-target cargo clippy -p tau-runtime --lib --tests -- -D warnings` passes.
- `cargo fmt --check` and `git diff --check` pass.
