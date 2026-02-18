# Spec #2461 - RED/GREEN conformance for G7 phase-3 lifecycle behavior

Status: Implemented
Milestone: specs/milestones/m78/index.md
Issue: https://github.com/njfio/Tau/issues/2461

## Problem Statement

Phase-3 lifecycle behavior (dedup + heartbeat execution) needs deterministic
conformance coverage to avoid silent over-deletion or heartbeat regressions.

## Scope

In scope:

- RED/GREEN conformance tests for C-01..C-04.
- Regression tests for deterministic canonical retention and heartbeat failure
  handling.

Out of scope:

- Non-lifecycle runtime features.

## Acceptance Criteria

- AC-1: RED tests fail before implementation for missing dedup/heartbeat paths.
- AC-2: GREEN tests pass after implementation.
- AC-3: Regression proves deterministic canonical retention and heartbeat
  fault-tolerant diagnostics.

## Conformance Cases

- C-01 (AC-1/AC-2): duplicate maintenance forgets non-canonical duplicate.
- C-02 (AC-3): canonical record remains stable across repeated runs.
- C-03 (AC-1/AC-2): heartbeat emits lifecycle success diagnostics.
- C-04 (AC-3): heartbeat emits lifecycle failure diagnostics without crash.
