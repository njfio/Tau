# Spec #2598 - Subtask: package conformance/live validation evidence for G16 hot-reload rollout

Status: Reviewed
Priority: P1
Milestone: M102
Parent: #2597

## Problem Statement
The G16 completion slice in #2597 changes runtime hot-reload behavior and requires reproducible verification evidence (tests, mutation, live smoke, process logs) before merge.

## Scope
- Re-run #2597 conformance suite.
- Run scoped quality gates and mutation-in-diff for touched paths.
- Run sanitized live validation smoke.
- Update closure artifacts (spec status, tasks evidence, issue logs).

## Out of Scope
- New feature work beyond #2597 ACs.

## Acceptance Criteria
- AC-1: #2597 conformance cases are reproducibly green.
- AC-2: mutation-in-diff reports zero missed mutants for #2597 changes.
- AC-3: sanitized live validation reports no failures.
- AC-4: issue/spec/task closure artifacts are complete and traceable.

## Conformance Cases
- C-01 (AC-1): mapped #2597 commands pass.
- C-02 (AC-2): `cargo mutants --in-diff <issue2597-diff> -p tau-coding-agent`.
- C-03 (AC-3): sanitized `scripts/dev/provider-live-smoke.sh` summary reports `failed=0`.
- C-04 (AC-4): issue closure comments + spec/task evidence updates are present.
