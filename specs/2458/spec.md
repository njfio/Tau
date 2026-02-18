# Spec #2458 - G7 memory lifecycle phase-3 orchestration

Status: Implemented
Milestone: specs/milestones/m78/index.md
Issue: https://github.com/njfio/Tau/issues/2458

## Problem Statement

G7 phase-1 and phase-2 shipped lifecycle metadata and maintenance APIs, but the
system still lacks near-duplicate lifecycle behavior and production heartbeat
execution wiring.

## Scope

In scope:

- Hierarchy/spec orchestration for lifecycle phase-3.
- Delivery of #2460 implementation and #2461 conformance evidence.

Out of scope:

- New embedding model dependencies.
- UI/dashboard lifecycle controls.

## Acceptance Criteria

- AC-1: Story/task/subtask hierarchy and milestone spec container exist.
- AC-2: Task #2460 lands near-duplicate lifecycle handling with deterministic
  tests.
- AC-3: Heartbeat lifecycle execution integration lands with diagnostics and
  conformance coverage.

## Conformance Cases

- C-01 (AC-1): `specs/2458..2461` and `specs/milestones/m78/index.md` exist.
- C-02 (AC-2): near-duplicate lifecycle conformance tests pass.
- C-03 (AC-3): heartbeat lifecycle conformance tests pass.
