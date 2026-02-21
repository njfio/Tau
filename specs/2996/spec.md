# Spec: Issue #2996 - Add panic/unsafe audit script with production-path guardrail evidence

Status: Implemented

## Problem Statement
Raw grep counts for `panic!` and `unsafe` are noisy and can mask whether production code is actually at risk. We need deterministic, repeatable audit output that separates test-heavy usage from non-test paths and pairs that with production-target guardrail checks.

## Acceptance Criteria

### AC-1 Deterministic audit script exists
Given the repo root,
When running `scripts/dev/audit-panic-unsafe.sh`,
Then the command prints stable, sorted panic/unsafe usage summaries and exits successfully.

### AC-2 Audit output separates path classes
Given panic/unsafe matches in the codebase,
When the audit script runs,
Then output includes total counts and a split between test-path and non-test-path occurrences for both `panic!` and `unsafe`.

### AC-3 Production-path guardrail checks are green
Given workspace production targets,
When running:
- `cargo clippy --workspace --all-features -- -A warnings -D clippy::panic`
- `cargo clippy --workspace --all-features -- -A warnings -D unsafe_code`
Then both commands pass.

### AC-4 Audit script behavior is conformance-tested
Given fixture files with known panic/unsafe usage,
When running the audit script against fixtures,
Then reported counts and classification match expected values.

## Scope

### In Scope
- Add `scripts/dev/audit-panic-unsafe.sh`.
- Add fixture/conformance test coverage for script behavior.
- Capture production guardrail command evidence.

### Out of Scope
- CI workflow changes (separate approval required).
- Repository-wide refactor of all test panic patterns.
- Semantic behavior changes to runtime systems.

## Conformance Cases
- C-01: Audit script exists and runs from repo root.
- C-02: Script output includes panic/unsafe totals and test/non-test split.
- C-03: Fixture-based script conformance test passes with expected counts.
- C-04: `clippy::panic` production-target guardrail passes.
- C-05: `unsafe_code` production-target guardrail passes.

## Success Metrics / Observable Signals
- `scripts/dev/test-audit-panic-unsafe.sh` passes.
- `scripts/dev/audit-panic-unsafe.sh` emits deterministic summary.
- Conformance cases C-01..C-05 pass.

## Approval Gate
P1 scope: spec authored/reviewed by agent; implementation proceeds and is flagged for human review in PR.
