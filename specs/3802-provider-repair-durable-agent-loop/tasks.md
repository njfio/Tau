# Tasks: Issue #3802 - Provider Repair Inside Durable Autonomous Coding Jobs

- [x] T1: Add RED runtime tests for provider repair success, diff success,
  malformed provider blocking, and issue-to-merge provider repair.
- [x] T2: Add runtime provider repair module with context capture, command
  adapter, JSON parsing, unified-diff preflight, and repair evidence.
- [x] T3: Integrate repair policy into durable job submit/run/replay and
  issue-to-merge.
- [x] T4: Add CLI flags for provider repair policy.
- [x] T5: Surface autonomous coding repair/recovery fields in `tau-unified`.
- [x] T6: Add deterministic scripts for issue-to-merge provider repair and
  gauntlet coverage.
- [x] T7: Update README and operator guide honestly.
- [x] T8: Run focused validation and record evidence here.

## Evidence

- `CARGO_TARGET_DIR=/tmp/rust_pi-3802-target cargo test -p tau-runtime spec_3802 -- --test-threads=1`
  passed: 4 tests.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3802-target cargo test -p tau-coding-agent --bin tau_autonomous_coding_job -- --test-threads=1`
  passed: binary compiled, 0 tests.
- `scripts/run/test-tau-unified.sh status_contract` passed.
- `scripts/run/test-tau-unified.sh` passed.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3802-target scripts/dev/test-autonomous-coding-issue-to-merge.sh`
  passed with `autonomous_coding_issue_to_merge=pass`.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3802-target scripts/dev/test-autonomous-coding-gauntlet.sh`
  passed with `{"suite":"autonomous_coding_gauntlet","passed":5,"failed":0}`.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3802-target cargo clippy -p tau-runtime --lib --tests -- -D warnings`
  passed.
- `CARGO_TARGET_DIR=/tmp/rust_pi-3802-target cargo clippy -p tau-coding-agent --bin tau_autonomous_coding_job -- -D warnings`
  passed.
- `cargo fmt --check`, shell `bash -n` for touched scripts, and
  `git diff --check` passed.
