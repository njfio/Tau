# Spec #2562 - Subtask: conformance + mutation + live-validation evidence for G2 phase-2

Status: Reviewed
Priority: P0
Milestone: M96
Parent: #2561

## Problem Statement
Task #2561 implemented profile-fed context compaction policy wiring, but merge gates require explicit evidence packaging (RED/GREEN, mutation, and live validation) for auditable closure.

## Scope
- Re-run and record conformance tests mapped to #2561 AC-1..AC-4.
- Produce mutation-in-diff summary for touched runtime/profile paths.
- Run deterministic live-validation smoke and capture summary output.
- Update issue process logs and checklist artifacts.

## Out of Scope
- Net-new runtime behavior changes beyond evidence-driven test hardening.
- Full multi-provider paid live matrix execution.

## Acceptance Criteria
- AC-1: Given #2561 conformance cases C-01..C-04, when verification runs, then all mapped tests pass and are recorded.
- AC-2: Given the #2561 code diff, when `cargo mutants --in-diff` runs on impacted crates, then missed mutants are zero (or explicitly resolved before closure).
- AC-3: Given local-safe live validation flow, when smoke runs with sanitized provider env/keys, then it completes with zero failures.
- AC-4: Given evidence is complete, when artifacts are updated, then `tasks/spacebot-comparison.md`, issue process log comments, and task status reflect completion.

## Conformance Cases
- C-01 (AC-1, conformance): `cargo test -p tau-onboarding spec_2561_` passes.
- C-02 (AC-1, conformance): `cargo test -p tau-agent-core runtime_turn_loop::tests::spec_2561_c03_context_monitor_snapshot_selects_expected_tiers -- --exact` and `...regression_2561_context_monitor_normalizes_unsorted_thresholds -- --exact` pass.
- C-03 (AC-2, mutation): `cargo mutants --in-diff /tmp/issue2561.diff -p tau-onboarding -p tau-agent-core` reports zero missed mutants.
- C-04 (AC-3, live validation): sanitized `./scripts/dev/provider-live-smoke.sh` run reports `failed=0`.

## Success Signals
- Evidence package is reproducible from commands in this spec.
- No unchecked/blank verification gates remain for #2561/#2562 closure.
