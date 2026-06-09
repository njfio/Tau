# Tasks: Hands-Off Issue-To-Merge Orchestration

- [x] T1: Add runtime tests for authorized issue-to-merge, auto-merge request,
  and no-authority blocking.
- [x] T2: Implement runtime issue-to-merge orchestration over existing job,
  PR-ready, and auto-merge paths.
- [x] T3: Add CLI `issue-to-merge` command.
- [x] T4: Add disposable shell integration proof.
- [x] T5: Update operator docs and README honestly.
- [x] T6: Run focused tests, script proof, format, clippy, and diff checks.

## Evidence

- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3801-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` passed 9 tests, including authorized issue-to-merge and no-authority block coverage.
- GREEN: `cargo fmt --check`
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3801-target cargo test -p tau-coding-agent --bin tau_autonomous_coding_job -- --test-threads=1`
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3801-target scripts/dev/test-autonomous-coding-issue-to-merge.sh` emitted `autonomous_coding_issue_to_merge=pass`.
- GREEN: `git diff --check`
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3801-target cargo clippy -p tau-runtime --lib --tests -- -D warnings`
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3801-target cargo clippy -p tau-coding-agent --bin tau_autonomous_coding_job -- -D warnings`
