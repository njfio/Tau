# Issue 1964 Spec

Status: Implemented

Issue: `#1964`  
Milestone: `#24`  
Parent: `#1659`

## Problem Statement

Trajectory adaptation exists, but there is no store-backed batch collection helper that
fetches rollout spans and converts them into deterministic trajectory batches for
training pipelines.

## Scope

In scope:

- add async trajectory collection helper over `TrainingStore` and rollout ids
- adapt spans with `SpansToTrajectories` and optional `TrajectoryWindowPolicy`
- return deterministic batch report with skipped rollouts and reasons

Out of scope:

- persistence schema changes
- CLI/dashboard surfaces
- optimizer/trainer execution semantics

## Acceptance Criteria

AC-1 (deterministic batch collection):
Given persisted spans for rollout ids,
when collection runs,
then helper returns deterministic trajectory batch output.

AC-2 (retry/requeue attempt coverage):
Given spans for multiple attempts of a rollout,
when collection runs,
then batch contains trajectories for each attempt.

AC-3 (window policy integration):
Given a window policy,
when collection runs,
then collected trajectories reflect truncate/pad rules.

AC-4 (unknown rollout fail closed):
Given unknown rollout ids,
when collection runs,
then helper returns deterministic unknown-rollout error.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given one rollout with valid spans, when collection runs, then batch has one valid trajectory and no skips. |
| C-02 | AC-2 | Integration | Given one rollout with two attempt span streams, when collection runs, then batch returns two trajectories. |
| C-03 | AC-3 | Conformance | Given 5-step spans and window size 3, when collection runs, then collected trajectory has 3 steps. |
| C-04 | AC-4 | Unit | Given unknown rollout id, when collection runs, then deterministic unknown-rollout error is returned. |

## Success Metrics

- store-backed trajectory collection becomes one helper call
- multi-attempt rollout trajectories are deterministically included
- window policy behavior composes cleanly with collection path
