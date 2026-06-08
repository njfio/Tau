# Tasks: Durable Autonomous Coding Job Loop

GitHub Issue: #3794
Status: Implemented

- [x] T1 (RED): Add failing runtime tests for durable submit, multi-file run,
  replay/resume, and status payload fields.
- [x] T2 (GREEN): Implement the autonomous coding job runtime and persisted
  record/status schema.
- [x] T3 (GREEN): Add the dedicated CLI binary with submit/run/status/recover
  and replay commands.
- [x] T4 (GREEN): Add a disposable-repo integration script that proves the
  full submit -> run/replay -> PR-ready loop.
- [x] T5 (VERIFY): Run focused cargo tests, CLI integration script, fmt, diff
  check, and clippy for touched crates.

## TDD Evidence

- RED: `CARGO_TARGET_DIR=/tmp/rust_pi-3794-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` failed with all four #3794 tests hitting `todo!("submit durable autonomous coding job")`.
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3794-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` passed after implementing submit, run/replay, recovery, and status snapshots.
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3794-target scripts/dev/test-autonomous-coding-job-loop.sh` passed and printed `autonomous_coding_job_loop=pass`.
- VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3794-target cargo test -p tau-coding-agent --bin tau_autonomous_coding_job -- --test-threads=1` passed.
- VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3794-target cargo clippy -p tau-runtime --lib --tests -- -D warnings` passed.
- VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3794-target cargo clippy -p tau-coding-agent --bin tau_autonomous_coding_job -- -D warnings` passed.
- VERIFY: `cargo fmt --check` and `git diff --check` passed.

## Test Tiers

| Tier | Status | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | PASS | `tau-runtime` autonomous coding job tests | |
| Property | N/A | | No randomized parser or invariant introduced in this slice |
| Contract/DbC | N/A | | No formal contract macro surface changed |
| Snapshot | N/A | | Status JSON uses field assertions rather than snapshots |
| Functional | PASS | Multi-file run to PR-ready | |
| Conformance | PASS | C-01..C-04 runtime and CLI coverage | |
| Integration | PASS | `scripts/dev/test-autonomous-coding-job-loop.sh` | |
| Fuzz | N/A | | No untrusted parser/fuzz target changed |
| Mutation | N/A | | Follow-up for release gate; this slice uses focused regression coverage |
| Regression | PASS | Recovery/replay checkpoint test | |
| Performance | N/A | | No hot path or benchmarked runtime changed |
