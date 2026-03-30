# Spec: Issue #3627 - Align plan_executor documentation with its actual reporting surface

Status: Implemented
Priority: P1
Milestone: M329
Parent: #3623

## Problem Statement
`crates/tau-orchestrator/src/plan_executor.rs` currently documents itself as a
"plan execution engine with parallel step scheduling" even though its public
surface only builds execution reports and detects deadlocks from an existing
plan state. That overclaims runtime capability and makes the orchestration
surface harder to reason about accurately.

## Scope
- Update module/item docs and adjacent tests so `plan_executor` is described as
  plan reporting/deadlock analysis support.
- Add regression coverage that locks the truth-in-advertising wording.

## Out of Scope
- Implementing a real step executor or parallel scheduler.
- Broader orchestrator feature work outside `plan_executor`.

## Acceptance Criteria
- AC-1: `plan_executor` no longer claims parallel scheduling or live execution
  behavior in module/item docs.
- AC-2: The public API documentation matches the implemented helper surface:
  execution reporting and deadlock detection.
- AC-3: Scoped `tau-orchestrator` tests lock the corrected semantics.

## Conformance Cases
- C-01 (AC-1, regression): module/item docs describe report/deadlock analysis
  rather than execution/scheduling.
- C-02 (AC-2, conformance): public types/functions are documented in language
  consistent with their actual behavior.
- C-03 (AC-3, conformance): scoped `tau-orchestrator` tests stay green.

## Success Signals
- No code-local documentation claims runtime capabilities that the module does
  not expose.
- Future contributors are less likely to mistake `plan_executor` for a real
  executor implementation.
