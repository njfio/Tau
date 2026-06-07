# Tasks

Status: Implemented

- [x] T1 (RED): Add failing runtime regressions for stale and fresh running manifests.
- [x] T2 (GREEN): Implement stuck running manifest recovery.
- [x] T3 (GREEN): Surface recovered stuck counters in jobs health payload.
- [x] T4 (VERIFY): Run focused tests and static checks.

## TDD Evidence

- RED: `CARGO_TARGET_DIR=/tmp/rust_pi-3775-target cargo test -p tau-runtime background_job_runtime -- --test-threads=1` failed before implementation with missing `BackgroundJobRuntime::recover_stuck_jobs` and missing `BackgroundJobHealthSnapshot::recovered_stuck_total`.
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3775-target cargo test -p tau-runtime background_job_runtime -- --test-threads=1` passed with stale/fresh stuck-recovery regressions.
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3775-target cargo test -p tau-tools regression_jobs_list_tool_recovers_stuck_running_manifest -- --test-threads=1` passed after wiring `jobs_list` to run the recovery sweep.
- VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3775-target cargo test -p tau-tools jobs_ -- --test-threads=1`, `cargo fmt --check`, `git diff --check`, and `CARGO_TARGET_DIR=/tmp/rust_pi-3775-target cargo clippy -p tau-runtime -p tau-tools -- -D warnings`.

## Test Tiers

| Tier | Status | Evidence | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | `tau-runtime` focused background-job runtime tests | |
| Property | N/A | | No parser/randomized invariant change in this slice |
| Contract/DbC | N/A | | No formal contract framework on this runtime surface |
| Snapshot | N/A | | No snapshot output changed |
| Functional | ✅ | `tau-tools` `jobs_list` recovery regression | |
| Conformance | ✅ | C-01/C-02/C-03 covered by focused runtime/tool tests | |
| Integration | ✅ | Tool regression verifies runtime recovery through `jobs_list` | |
| Fuzz | N/A | | No untrusted parser/input format change |
| Mutation | N/A | | Not run for this focused product slice; follow-up if this becomes a release-critical gate |
| Regression | ✅ | Stale/fresh manifest recovery and tool-loop recovery regressions | |
| Performance | N/A | | Recovery scans persisted manifests; no benchmarked hot path changed |
