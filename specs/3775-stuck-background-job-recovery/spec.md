# Issue #3775: Stuck Background Job Recovery

Status: Implemented

## Problem

Tau background jobs recover persisted `running` manifests on runtime restart, but there is no callable runtime sweep for jobs that remain `running` past their timeout while the runtime is alive or after a worker has stopped making progress. That keeps stuck-job recovery out of the actual product loop.

## Scope

In:
- Add runtime behavior that detects stale `running` background-job manifests and requeues them.
- Persist recovery events, reason codes, and health counters.
- Preserve existing restart recovery behavior.

Out:
- No new scheduler crate or dependency.
- No full crash-resume/replay claim.
- No gateway or dashboard redesign.

## Acceptance Criteria

### AC-1: Stuck Running Jobs Are Requeued

Given a persisted background-job manifest is `running` and its last activity is older than its effective timeout, when the recovery sweep runs, then the job is requeued with a stuck-recovery reason code and can execute to completion.

### AC-2: Fresh Running Jobs Are Not Requeued

Given a persisted background-job manifest is `running` but has not exceeded its effective timeout, when the recovery sweep runs, then the job remains `running` and no recovery counter is incremented.

### AC-3: Recovery Is Operator Visible

Given a stuck job is recovered, when health is inspected through the jobs runtime/tool output, then recovered-stuck counters and reason codes show the recovery.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Regression | stale `running` manifest with executable command | recovery sweep runs | report shows one recovered job, event log includes stuck-recovery reason, job succeeds |
| C-02 | AC-2 | Regression | fresh `running` manifest | recovery sweep runs | report shows zero recovered jobs and manifest remains `running` |
| C-03 | AC-3 | Functional | recovered stuck job | health is inspected | `recovered_stuck_total` and reason codes include the recovery |
