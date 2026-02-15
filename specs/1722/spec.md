# Issue 1722 Spec

Status: Implemented

Issue: `#1722`  
Milestone: `#23`  
Parent: `#1701`

## Problem Statement

M23 requires doc quotas to be operationalized by owner domain and review
cadence. Existing target thresholds define percentages but do not encode domain
ownership, cadence intervals, checkpoint dates, or explicit escalation handling.

## Scope

In scope:

- assign owner domains to quota crates
- define review cadence by owner domain
- publish checkpoint review schedule
- define missed-target escalation path
- reflect metadata in scorecard/targets docs surfaces

Out of scope:

- assigning individual human owners
- CI workflow modifications

## Acceptance Criteria

AC-1 (owner/cadence metadata):
Given allocation artifacts,
when reviewing quota rows,
then each row includes owner domain and review cadence.

AC-2 (checkpoint and escalation policy):
Given M23 quota plan,
when reviewing policy,
then checkpoint dates and missed-target escalation are explicit.

AC-3 (docs surfaces alignment):
Given scorecard/targets docs,
when read,
then owner-domain cadence and allocation plan references are present.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given plan JSON, when reading `quota_allocations`, then owner/cadence fields exist and match domain defaults. |
| C-02 | AC-2 | Conformance | Given plan JSON, when reading checkpoints and escalation arrays, then required fields and steps are present. |
| C-03 | AC-3 | Integration | Given scorecard/targets/docs index, when checked, then allocation plan references and cadence metadata are discoverable. |
| C-04 | AC-1, AC-2, AC-3 | Regression | Given contract test, when run, then owner/cadence/checkpoint/docs invariants remain valid. |

## Success Metrics

- owner-domain quotas, cadence, checkpoints, and escalation policy are explicit
- docs surfaces reference the same allocation contract
- regression tests fail fast on policy drift
