# Issue 1996 Spec

Status: Implemented

Issue: `#1996`  
Milestone: `#24`  
Parent: `#1702`

## Problem Statement

M24 exit evidence exists in separate benchmark, safety, operations, and runbook
artifacts, but there is no deterministic bundle/export/validator flow that
packages this evidence into one replayable payload for gate verification.

## Scope

In scope:

- add typed M24 RL gate evidence bundle model with nested sections
- add deterministic bundle builder helper
- add deterministic export helper and replay validator helper
- enforce required top-level `benchmark`, `safety`, `operations`, and `runbooks`
  sections

Out of scope:

- executing live benchmark/safety/recovery runs
- dashboard rendering
- CI workflow wiring

## Acceptance Criteria

AC-1 (deterministic builder):
Given valid benchmark/safety/operations/runbook evidence input,
when bundle builder runs,
then deterministic nested sections and pass/fail signals are preserved.

AC-2 (deterministic export):
Given valid bundle and output directory,
when export runs,
then deterministic path and bytes-written summary are returned.

AC-3 (validator pass path):
Given exported bundle JSON,
when validator runs,
then payload is accepted with required top-level sections.

AC-4 (validator fail closed):
Given malformed, non-object, or missing-section bundle payloads,
when validator runs,
then deterministic validation errors are returned.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given valid evidence input, when builder runs, then bundle contains deterministic `benchmark`/`safety`/`operations`/`runbooks` sections and pass flags. |
| C-02 | AC-2 | Conformance | Given valid bundle and output directory, when export runs, then deterministic filename is written and bytes_written > 0. |
| C-03 | AC-3 | Unit | Given exported bundle payload, when validator runs, then required top-level sections are accepted as objects. |
| C-04 | AC-4 | Regression | Given malformed/non-object/missing-section payloads, when validator runs, then deterministic fail-closed errors are returned. |

## Success Metrics

- one deterministic artifact can represent M24 gate evidence end-to-end
- replay validation fails closed on malformed or incomplete evidence payloads
- export outputs remain machine-readable for automation and audits
