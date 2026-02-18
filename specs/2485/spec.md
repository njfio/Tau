# Spec #2485 - G16 hot-reload config phase-2 orchestration

Status: Implemented

## Problem Statement
`G16` remains only partially implemented after M79. Tau can hot-reload heartbeat interval via a polled JSON sidecar, but it does not yet provide notify-driven profile-policy TOML watching with lock-free config swaps.

## Acceptance Criteria
### AC-1 Phase-2 scope is explicit and bounded
Given M83 execution, when implementation runs, then scope is limited to runtime heartbeat profile-policy watcher + atomic config swap behavior.

### AC-2 Child issues provide complete conformance traceability
Given #2486/#2487/#2488, when work completes, then AC-to-conformance-to-test mappings and RED/GREEN evidence are present.

## Scope
In scope:
- M83 issue hierarchy and binding artifacts.
- `tau-runtime` heartbeat profile-policy watcher + atomic reload slice.

Out of scope:
- Full profile-wide hot reload.
- Prompt template watcher behavior.

## Conformance Cases
- C-01 (AC-1, governance): milestone index + child specs define bounded phase-2 scope.
- C-02 (AC-2, governance): #2487/#2488 include conformance tests and RED/GREEN evidence.

## Success Metrics
- M83 closes with all child issues closed.
- #2487 AC matrix has no failing entries.
