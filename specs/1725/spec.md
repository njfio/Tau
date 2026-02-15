# Issue 1725 Spec

Status: Implemented

Issue: `#1725`  
Milestone: `#24`  
Parent: `#1666`

## Problem Statement

The RL experience collector needs deterministic load validation to prove it
remains stable under bursty concurrent worker traffic and does not silently drop
rollouts.

## Scope

In scope:

- add a concurrent load harness for training runner + store
- capture throughput/latency metrics during harness execution
- assert no silent drops (all enqueued rollouts reach terminal state)

Out of scope:

- external benchmarking infrastructure
- distributed multi-node load generation
- dependency additions

## Acceptance Criteria

AC-1 (concurrent burst stability):
Given a burst of queued rollouts and multiple workers,
when the collector harness runs,
then all rollouts complete with no hangs or silent drops.

AC-2 (metrics observability):
Given the load harness execution,
when the run completes,
then throughput and elapsed timing metrics are emitted.

AC-3 (repeatable regression guard):
Given future collector changes,
when the harness regression test runs,
then pass/fail behavior remains deterministic for no-drop guarantees.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Integration | Given N rollouts and M workers, when harness executes, then succeeded rollouts count equals N. |
| C-02 | AC-2 | Functional | Given harness completion, when metrics are collected, then elapsed milliseconds and throughput are non-zero. |
| C-03 | AC-3 | Regression | Given repeated harness execution, when verifying terminal statuses, then there are zero failed/cancelled rollouts from silent drop paths. |

## Success Metrics

- deterministic collector load harness is executable via test and script
- run outputs include throughput and elapsed duration
- no-drop invariant remains locked by regression checks
