# Tasks: Issue #3754 - Close Tau autonomous harness integration gaps

Status: Implemented

- [x] T1 (SPEC): create #3754 issue/spec/plan/tasks and link M334.
- [x] T2 (RED): add failing `tau-agent-core` tests for deterministic mission
      harness and M334 benchmark proof execution.
- [x] T3 (GREEN): implement `tau-agent-core` harness proof runner.
- [x] T4 (RED): add failing `tau-coding-agent` tests for mission-linked
      self-modification dry-run and approved safe apply.
- [x] T5 (GREEN): implement `tau-coding-agent` mission self-improvement
      adapter and apply helper.
- [x] T6 (RED): add failing operator binary test for canonical benchmark proof
      emission.
- [x] T7 (GREEN): implement `tau_agent_harness` binary.
- [x] T8 (VERIFY): run scoped tests, shell benchmark validator, fmt, clippy,
      PR checks, and issue update.

## Verification Evidence

- RED: `cargo test -p tau-agent-core --test mission_harness` failed before
  implementation with unresolved `tau_agent_core::mission_harness`.
- GREEN: `cargo test -p tau-agent-core --test mission_harness` passed
  (2 tests).
- GREEN: `cargo test -p tau-coding-agent --test mission_self_improvement`
  passed (3 tests).
- GREEN: `cargo test -p tau-coding-agent --test harness_benchmark_bin` passed
  (1 test).
- REGRESSION: `cargo test -p tau-coding-agent --test self_mod_dry_run_bin --test synthesize_then_propose_chain`
  passed (5 tests).
- BENCHMARK: `scripts/dev/test-m334-tranche-one-autonomy-benchmark.sh`
  passed, including the `tau_agent_harness` proof run.
- FORMAT: `cargo fmt --check -p tau-agent-core -p tau-coding-agent` passed.
- CLIPPY: `cargo clippy -p tau-agent-core -p tau-coding-agent --all-targets -- -D warnings`
  passed.
