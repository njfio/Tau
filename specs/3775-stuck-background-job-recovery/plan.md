# Plan

Status: Implemented

1. Add failing `tau-runtime` tests that seed stale and fresh `running` job manifests.
2. Add a `BackgroundJobRuntime::recover_stuck_jobs` sweep with a small report type.
3. Persist `job_recovered_after_stuck_timeout` events and update health counters/reason codes.
4. Add the new health field to the jobs tool payload.
5. Run focused runtime/tool tests and update evidence.

## Affected Modules

- `crates/tau-runtime/src/background_jobs_runtime.rs`
- `crates/tau-tools/src/tools/jobs_tools.rs`
- `crates/tau-tools/src/tools/runtime_helpers.rs`
- `crates/tau-tools/src/tools/tests.rs`
- `docs/guides/background-jobs-ops.md`
- `specs/3775-stuck-background-job-recovery/*`

## Risks

- Requeueing a legitimately running process can duplicate work. Mitigation: only recover manifests whose last activity exceeds `effective_timeout_ms` and skip cancellation-requested jobs.
- Health counters can drift from disk. Mitigation: recompute running count from manifests during recovery.
- This could be mistaken for full crash-resume. Mitigation: keep scope limited to background-job manifest recovery.
