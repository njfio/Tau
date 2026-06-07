# Issue #3777: Background Job Recovery Watchdog

Status: Implemented

## Problem

Issue #3775 made stuck background-job recovery real, but still reactive: the
operator or agent has to inspect jobs before stale `running` manifests are
requeued. A usable autonomous runtime needs the active jobs runtime to keep
checking for stuck manifests while non-terminal work exists.

## Scope

In:
- Add a configurable recovery watchdog to `BackgroundJobRuntime`.
- Start the watchdog from existing job runtime paths while non-terminal jobs
  exist.
- Reuse the existing `job_recovered_after_stuck_timeout` reason code,
  `recovered_stuck_total` counter, event log, and worker scheduling behavior.
- Avoid requeueing manifests that are still fresh or currently owned by an
  active worker loop.

Out:
- No new scheduler crate or dependency.
- No claim of full mission replay/crash-resume.
- No dashboard redesign.

## Acceptance Criteria

### AC-1: Watchdog Recovers Stale Running Jobs

Given a jobs runtime has watchdog recovery enabled and a persisted job manifest
is `running` beyond its effective timeout while no worker is active, when the
watchdog tick runs, then the job is requeued, scheduled, and can execute to
completion.

### AC-2: Watchdog Does Not Duplicate Active Worker Jobs

Given the runtime worker is active, when the watchdog tick runs, then it does
not requeue a worker-owned `running` manifest even if the manifest appears old.

### AC-3: Recovery Remains Operator Visible

Given the watchdog recovers a stuck job, when health is inspected, then
`recovered_stuck_total` and `job_recovered_after_stuck_timeout` show the
recovery through the existing health surface.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Regression | stale `running` manifest and idle worker | watchdog tick runs | manifest is requeued and job succeeds |
| C-02 | AC-2 | Regression | active worker loop | watchdog tick runs | recovery is skipped and no duplicate queue entry is created |
| C-03 | AC-3 | Functional | watchdog-recovered job | health is inspected | stuck recovery counter and reason code are visible |

## Success Metrics

- Active background-job runtimes do not require a `jobs_list` call to recover
  stale `running` manifests.
- Existing explicit recovery behavior from #3775 remains compatible.
