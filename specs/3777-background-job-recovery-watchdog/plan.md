# Plan

Status: Implemented

1. Add failing `tau-runtime` regressions for watchdog recovery and active-worker
   skip behavior.
2. Extend `BackgroundJobRuntimeConfig` with a recovery watchdog poll interval.
3. Add a bounded watchdog loop that wakes while non-terminal jobs exist, skips
   recovery when the worker is active, and reuses `recover_stuck_jobs`.
4. Enable the watchdog from the jobs tool runtime construction path.
5. Update ops docs/spec evidence and run focused runtime/tool validation.

## Affected Modules

- `crates/tau-runtime/src/background_jobs_runtime.rs`
- `crates/tau-tools/src/tools/runtime_helpers.rs`
- `docs/guides/background-jobs-ops.md`
- `specs/3777-background-job-recovery-watchdog/*`

## Risks

- Risk: watchdog duplicates work while a worker is still executing a long job.
  Mitigation: watchdog skips recovery whenever the runtime worker loop is active.
- Risk: watchdog tasks keep one-shot processes alive.
  Mitigation: watchdog exits once no non-terminal manifests remain.
- Risk: this is overstated as full crash replay.
  Mitigation: scope and docs keep the claim limited to background-job manifest
  recovery.
