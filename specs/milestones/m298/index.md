# M298 - Tau E2E + Ops Dashboard Integrated Delivery

Status: Active

## Context
M298 starts the next delivery cycle from:
- `specs/tau-e2e-testing-prd.md`
- `specs/tau-ops-dashboard-prd.md`

The milestone focus is to turn PRD intent into a test-first, executable first slice that binds end-to-end runtime verification and operator dashboard behavior.

## Issue Hierarchy
- Epic: #3446
- Story: #3447
- Task (wave-1 delivery): #3448
- Subtask (workspace hygiene baseline): #3449

## Wave-1 Scope
- Define and implement a bounded `TauE2eHarness` foundation with first integrated scenarios.
- Enforce ops dashboard live data/control conformance against gateway/runtime artifacts.
- Preserve strict AGENTS contract gates (spec -> plan -> tasks -> tests-first implementation -> verification).

## Exit Criteria
- #3448 marked `Status: Implemented` with ACs and conformance cases passing.
- Required test-tier matrix completed with no blank cells and justified N/A entries.
- PR gate evidence includes RED/GREEN/REGRESSION output excerpts.
- #3449 closed or explicitly deferred with linked follow-up issue.
