# Issue 1998 Spec

Status: Implemented

Issue: `#1998`  
Milestone: `#24`  
Parent: `#1702`

## Problem Statement

M24 evidence can now be bundled deterministically, but there is no deterministic
gate evaluator that converts bundle sections into a single pass/fail decision
with explicit reason codes for exit checks.

## Scope

In scope:

- add `M24RLGateExitDecision` model
- add deterministic evaluator over `M24RLGateEvidenceBundle`
- add machine-readable decision JSON projection
- fail closed on missing runbook references

Out of scope:

- executing live benchmark/safety/operations runs
- CI workflow enforcement
- dashboard visualization

## Acceptance Criteria

AC-1 (pass path):
Given bundle with passing benchmark/safety/operations and non-empty runbook refs,
when evaluator runs,
then decision passes with no reason codes.

AC-2 (failing sections):
Given bundle with failing benchmark/safety/operations sections,
when evaluator runs,
then deterministic reason codes identify each failing section.

AC-3 (machine-readable output):
Given exit decision,
when serialized,
then JSON payload is machine-readable and includes pass flag and reason codes.

AC-4 (runbook fail closed):
Given bundle with blank runbook refs,
when evaluator runs,
then deterministic runbook-missing reason codes are returned.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given fully passing bundle, when evaluator runs, then `pass=true` and `reason_codes=[]`. |
| C-02 | AC-2 | Unit | Given section failures, when evaluator runs, then deterministic per-section reason codes are emitted. |
| C-03 | AC-3 | Conformance | Given exit decision, when serialized, then JSON has machine-readable `pass` + `reason_codes` fields. |
| C-04 | AC-4 | Regression | Given blank runbook refs, when evaluator runs, then fail-closed runbook reason codes are emitted. |

## Success Metrics

- M24 exit status is computed by deterministic code, not manual interpretation
- failure diagnostics are explicit and machine-readable for automation
