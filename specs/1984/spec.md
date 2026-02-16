# Issue 1984 Spec

Status: Accepted

Issue: `#1984`  
Milestone: `#24`  
Parent: `#1662`

## Problem Statement

Summary manifests and summary quality decisions now exist as separate helpers,
but there is no single deterministic summary gate report payload bundling both
outputs for automation and audit consumers.

## Scope

In scope:

- add summary gate report model embedding summary manifest + quality decision
- add builder helper that derives quality decision from summary manifest + policy
- add machine-readable JSON projection for summary gate reports

Out of scope:

- CI workflow wiring
- dashboard rendering
- remote transport/storage

## Acceptance Criteria

AC-1 (deterministic combined report):
Given a summary manifest and summary quality policy,
when report builder runs,
then report includes deterministic summary counters and quality decision fields.

AC-2 (reason propagation):
Given a failing summary/policy outcome,
when report is built,
then summary quality reason codes are preserved in output.

AC-3 (machine-readable serialization):
Given a summary gate report,
when serialized,
then JSON payload exposes nested `summary` and `quality` objects.

AC-4 (invalid policy fail closed):
Given an invalid summary quality policy (`max_fail_ratio` or
`max_invalid_file_ratio` outside `[0,1]`),
when report builder runs,
then deterministic validation error is returned.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given summary(pass=2, fail=0, invalid=0) and permissive policy, when report builds, then pass=true and counters match summary. |
| C-02 | AC-2 | Integration | Given summary(pass=0, fail=2, invalid=1), when report builds, then quality reason codes include threshold-failure reasons. |
| C-03 | AC-3 | Conformance | Given built report, when serialized, then payload contains `summary` and `quality` objects with machine-readable fields. |
| C-04 | AC-4 | Unit | Given policy with `max_fail_ratio=1.5`, when report builds, then deterministic out-of-range validation error is returned. |

## Success Metrics

- one helper returns full summary+quality gate report for operators
- no data loss between summary counters and quality-decision output
- JSON output is directly consumable by automation without ad hoc parsing
