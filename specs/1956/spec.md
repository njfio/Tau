# Issue 1956 Spec

Status: Implemented

Issue: `#1956`  
Milestone: `#24`  
Parent: `#1659`

## Problem Statement

`TrainingRunner::run` currently exits on any `process_once` poll-path error. Transient
store failures can terminate collection workers and reduce rollout throughput.

## Scope

In scope:

- add bounded transient retry backoff controls to runner config
- apply retry/backoff only to poll-path `process_once` failures
- reset retry/backoff state after successful poll cycle

Out of scope:

- heartbeat/reassignment error policy changes
- store-layer retry semantics
- distributed circuit-breaker/orchestration policy

## Acceptance Criteria

AC-1 (transient error recovery):
Given a transient `process_once` error,
when runner poll tick executes,
then runner does not exit and retries after backoff.

AC-2 (bounded backoff):
Given consecutive poll failures,
when retry delay is computed,
then delay grows exponentially and is capped at configured max.

AC-3 (success resets backoff):
Given one or more poll failures followed by success,
when next failure occurs,
then retry delay restarts from initial backoff.

AC-4 (config safety):
Given zero or invalid backoff durations,
when runner is constructed,
then config validation fails with deterministic error.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Integration | Given store dequeue fails once then succeeds, when runner executes, then rollout still completes succeeded. |
| C-02 | AC-2 | Unit | Given increasing failure counts, when computing retry delay, then result doubles up to max bound. |
| C-03 | AC-3 | Regression | Given fail->success->fail sequence, when delay computed, then second failure uses initial delay. |
| C-04 | AC-4 | Unit | Given invalid retry config, when validation runs, then deterministic config error returns. |

## Success Metrics

- transient poll errors no longer kill active runner loop
- retry delay behavior is deterministic and bounded
- recovery behavior is locked with integration/regression tests
