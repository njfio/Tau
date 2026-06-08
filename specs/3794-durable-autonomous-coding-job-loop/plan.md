# Plan: Durable Autonomous Coding Job Loop

GitHub Issue: #3794
Status: Implemented

## Approach

Add a new `tau-runtime` autonomous coding job module that composes existing
building blocks:

- `CodingMissionState` and `CodingMissionRunner` remain the source of truth for
  branch, verifier, resume, commit, and PR-ready behavior.
- `BackgroundJobRuntime` remains the durable queue, timeout, restart-recovery,
  and stuck-recovery substrate.
- The new autonomous job record links those two records and stores the operator
  summary needed to inspect or replay work.

Add a small `tau-autonomous-coding-job` binary in `tau-coding-agent` for
operator/test use:

- `submit`: create mission/job records and enqueue a background job that calls
  the same binary's `run` command.
- `run`: run or replay a mission until it reaches `pr_ready` or a true blocker.
- `status`: print the persisted operator status JSON.
- `recover`: invoke the existing background-job stuck recovery sweep and refresh
  the autonomous status.
- `replay`: explicitly resume a mission checkpoint without requiring a queue
  event.

## Affected Modules

- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
- `crates/tau-runtime/src/lib.rs`
- `crates/tau-coding-agent/src/bin/tau_autonomous_coding_job.rs`
- `scripts/dev/test-autonomous-coding-job-loop.sh`
- `specs/3794-durable-autonomous-coding-job-loop/*`

## Risks And Mitigations

- Risk: duplicating mission state logic. Mitigation: delegate all branch,
  verifier, edit, commit, and PR-ready work to `CodingMissionRunner` and
  `CodingMissionState`.
- Risk: overstating arbitrary autonomy. Mitigation: require verifier commands
  and bounded controlled edits in this slice; status reports PR-ready handoff,
  not automatic merge.
- Risk: background worker races in tests. Mitigation: runtime unit tests call
  the autonomous runner directly; CLI proof uses a disposable repo and explicit
  status polling.

## Interfaces

- Autonomous job JSON records live under
  `<state-dir>/autonomous-coding-jobs/<job-id>.json`.
- Operator status snapshots live under
  `<state-dir>/autonomous-coding-jobs/<job-id>.status.json`.
- Mission state remains under `<state-dir>/coding-missions/<mission-id>.json`.
- Background job state remains under the configured background jobs state dir.

## ADR

No new ADR: this composes existing mission and background-job architecture and
does not introduce new dependencies, protocols, or scheduler semantics.
