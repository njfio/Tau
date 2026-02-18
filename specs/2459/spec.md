# Spec #2459 - implement G7 near-duplicate lifecycle + heartbeat execution

Status: Implemented
Milestone: specs/milestones/m78/index.md
Issue: https://github.com/njfio/Tau/issues/2459

## Problem Statement

Tau memory lifecycle still allows near-duplicate active memories to accumulate,
and lifecycle maintenance is not invoked by runtime heartbeat in production.

## Scope

In scope:

- Near-duplicate detection during lifecycle maintenance.
- Deterministic canonical record retention and duplicate soft-forget behavior.
- Heartbeat-triggered lifecycle maintenance execution.
- Heartbeat diagnostics/reason-code capture for lifecycle runs and failures.

Out of scope:

- Local embedding model upgrades.
- New dashboard surfaces.

## Acceptance Criteria

- AC-1: Lifecycle maintenance can detect near-duplicate active records by
  embedding similarity threshold.
- AC-2: Duplicate handling soft-forgets non-canonical duplicates while keeping
  canonical record active.
- AC-3: Runtime heartbeat executes lifecycle maintenance for configured memory
  store root(s) and reports lifecycle counters.
- AC-4: Lifecycle maintenance failures do not crash heartbeat cycles and are
  surfaced via diagnostics/reason codes.

## Conformance Cases

- C-01 (AC-1, integration): similar-memory pair above threshold is detected.
- C-02 (AC-2, functional): duplicate non-canonical memory is forgotten and
  canonical memory remains active.
- C-03 (AC-3, integration): heartbeat cycle executes lifecycle maintenance and
  emits lifecycle diagnostics/reason code.
- C-04 (AC-4, regression): invalid lifecycle config or store load error is
  captured as heartbeat diagnostics without scheduler failure.
