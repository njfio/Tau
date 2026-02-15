# Issue 1665 Spec

Status: Implemented

Issue: `#1665`  
Milestone: `#24`  
Parent: `#1658`

## Problem Statement

Tau now has RL trajectory schema types, but lacks an adapter that maps existing
training spans into validated RL trajectory payloads. This blocks downstream PPO
and collector work from consuming current span telemetry.

## Scope

In scope:

- add span-to-trajectory adapter(s) for existing `TrainingSpan` payloads
- handle partial telemetry with deterministic fallbacks
- emit deterministic adapter errors for unsupported/empty input
- add unit/functional/regression tests for adapter behavior

Out of scope:

- PPO optimization math
- runner/store queue concurrency changes
- checkpoint persistence features

## Acceptance Criteria

AC-1 (trajectory mapping):
Given ordered training spans,
when adapter runs,
then it outputs one or more `EpisodeTrajectory` values with ordered
`TrajectoryStep` entries.

AC-2 (schema validity):
Given adapter output,
when `EpisodeTrajectory::validate` runs,
then all generated trajectories pass validation.

AC-3 (partial telemetry handling):
Given spans missing explicit observation/action/reward fields,
when adapter runs,
then fallback observation/action metadata is generated without panics.

AC-4 (deterministic failure reasons):
Given unsupported adapter input (for example empty span set),
when adapter runs,
then it fails with stable, explicit error text.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given multi-span trace, when adapted, then trajectory count and ordered step indexes are deterministic. |
| C-02 | AC-2 | Conformance | Given adapter output, when validated, then all trajectories pass `validate()`. |
| C-03 | AC-3 | Regression | Given spans missing structured fields, when adapted, then fallback state/action values are emitted. |
| C-04 | AC-4 | Regression | Given empty span input, when adapted, then adapter returns deterministic error text. |

## Success Metrics

- RL consumers can use existing span telemetry without schema translation glue
- adapter output is validation-clean and test-covered
