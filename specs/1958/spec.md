# Issue 1958 Spec

Status: Implemented

Issue: `#1958`  
Milestone: `#24`  
Parent: `#1659`

## Problem Statement

The runner now retries transient poll failures, but retry behavior is not emitted as
structured metrics in rollout telemetry. Operators cannot audit how much backoff
was required before successful rollout execution.

## Scope

In scope:

- track retry recovery context in poll loop (consecutive failures + accumulated backoff)
- emit deterministic retry metrics as `reward.emit` spans when a rollout is processed
  after transient poll failures
- preserve existing rollout status and attempt semantics

Out of scope:

- external metrics backends (Prometheus/OTel)
- schema changes to training store
- non-poll-path retry instrumentation

## Acceptance Criteria

AC-1 (retry metrics emitted on recovery):
Given transient poll failures before a successful dequeue,
when the rollout completes,
then telemetry includes retry-failure-count and accumulated-backoff metrics.

AC-2 (deterministic metric values):
Given configured retry backoff and known failure count,
when recovery occurs,
then emitted metric values match deterministic expected totals.

AC-3 (no-noise on clean runs):
Given no transient poll failures,
when rollout completes,
then retry-recovery metrics are not emitted.

AC-4 (no behavior regression):
Given existing runner/trainer workloads,
when metrics instrumentation is enabled,
then rollout outcomes remain unchanged.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Integration | Given dequeue fails once then succeeds, when rollout completes, then `reward.emit` includes `runner.poll_retry_failures_before_rollout` and `runner.poll_retry_backoff_ms_before_rollout`. |
| C-02 | AC-2 | Conformance | Given 3 transient dequeue failures with initial=5ms, max=20ms, when rollout completes, then backoff metric is 35 and failure metric is 3. |
| C-03 | AC-3 | Functional | Given no poll failures, when rollout succeeds, then no retry-recovery metric rewards are emitted. |
| C-04 | AC-4 | Regression | Given existing runner/trainer suites, when tests run, then all prior success/failure semantics remain green. |

## Success Metrics

- retry-recovery telemetry becomes queryable from persisted spans
- metric values are deterministic and auditable
- no regression in existing runner/trainer behavior
