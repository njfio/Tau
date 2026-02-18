# Spec #2451 - RED/GREEN conformance coverage for G7 lifecycle metadata slice

Status: Implemented
Milestone: specs/milestones/m76/index.md
Issue: https://github.com/njfio/Tau/issues/2451

## Problem Statement

The G7 phase-1 runtime slice must land with deterministic conformance tests so
lifecycle metadata and forgotten filtering cannot silently regress.

## Scope

In scope:

- RED/GREEN tests for lifecycle defaults and soft-delete behavior.
- Tool-level conformance for `memory_delete`.
- Regression checks for active-record behavior.

Out of scope:

- Production feature design beyond parent task #2450.

## Acceptance Criteria

- AC-1: RED tests fail before lifecycle implementation wiring.
- AC-2: GREEN tests pass after runtime/tool implementation.
- AC-3: Regression tests prove active records remain visible.

## Conformance Cases

- C-01 (AC-1/AC-2): lifecycle default serde + access-touch integration tests.
- C-02 (AC-1/AC-2): `memory_delete` success + filtering tests.
- C-03 (AC-3): active-record read/search/list/tree still return expected data.
