# Issue 1998 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): add failing tests for pass path, failing-section reason codes,
machine-readable decision JSON projection, and runbook fail-closed behavior.

T2: add `M24RLGateExitDecision` model with JSON projection.

T3: add `evaluate_m24_rl_gate_exit(bundle)` deterministic evaluator.

T4: run scoped verification and map AC-1..AC-4 to C-01..C-04.

## Tier Mapping

- Unit: section-failure reason code behavior
- Functional: fully passing bundle decision path
- Conformance: machine-readable decision JSON projection
- Regression: blank runbook fail-closed behavior
