# Tasks

Status: Implemented

- [x] T1 (RED): Add failing watchdog recovery and active-worker skip tests.
- [x] T2 (GREEN): Implement bounded recovery watchdog scheduling.
- [x] T3 (GREEN): Enable watchdog in the jobs tool runtime path.
- [x] T4 (VERIFY): Run focused runtime/tool tests, fmt, diff check, and clippy.

## TDD Evidence

- RED: `CARGO_TARGET_DIR=/tmp/rust_pi-3777-target cargo test -p tau-runtime watchdog -- --test-threads=1` failed before implementation because `BackgroundJobRuntimeConfig` had no `stuck_recovery_poll_ms` field and the watchdog methods did not exist.
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3777-target cargo test -p tau-runtime watchdog -- --test-threads=1` passed after implementation.
- VERIFY:
  - `CARGO_TARGET_DIR=/tmp/rust_pi-3777-target cargo test -p tau-runtime background_job_runtime -- --test-threads=1`
  - `CARGO_TARGET_DIR=/tmp/rust_pi-3777-target cargo test -p tau-tools jobs_ -- --test-threads=1`
  - `cargo fmt --check`
  - `git diff --check`
  - `scripts/dev/roadmap-status-sync.sh --check --quiet`
  - `CARGO_TARGET_DIR=/tmp/rust_pi-3777-target cargo clippy -p tau-runtime -p tau-tools -- -D warnings`

## Test Tiers

| Tier | Status | Tests | Notes |
| --- | --- | --- | --- |
| Unit | ✅ | `cargo test -p tau-runtime background_job_runtime` | Runtime regression suite validates public behavior. |
| Property | N/A | N/A | No new parser/serializer invariant introduced. |
| Contract/DbC | N/A | N/A | No new contract macro surface introduced. |
| Snapshot | N/A | N/A | No snapshot output changed. |
| Functional | ✅ | watchdog recovery tests | Covers AC-1 and AC-3. |
| Conformance | ✅ | watchdog recovery tests | Covers C-01..C-03. |
| Integration | ✅ | `cargo test -p tau-tools jobs_` | Verifies tool runtime construction. |
| Fuzz | N/A | N/A | No untrusted parser or fuzz target changed. |
| Mutation | N/A | N/A | Narrow runtime watchdog slice; follow-up mutation reserved for larger release gate. |
| Regression | ✅ | watchdog regression tests | Locks stale-manifest recovery and active-worker skip. |
| Performance | N/A | N/A | Poll loop is bounded/configurable; no hotspot benchmark changed. |
