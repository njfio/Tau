# Tasks: Issue #3300 - close Review #37 curriculum/meta-cognition gaps

- [x] T1 (RED): add failing conformance/regression tests for meta-cognition span fields, curriculum sample weighting, and trend classification.
- [x] T2 (GREEN): implement task-category inference and confidence/meta-cognition span attributes.
- [x] T3 (GREEN): implement curriculum-weighted APO sample capping with weakest-category diagnostic reporting.
- [x] T4 (GREEN): update Review #37 OpenTelemetry status text to match implemented state.
- [x] T5 (VERIFY): run targeted conformance tests for C-01..C-03 and existing optimizer regression coverage.
- [x] T6 (VERIFY): run `cargo fmt --check` and `cargo clippy -p tau-coding-agent --no-deps -- -D warnings`.
- [x] T7 (VERIFY): run `cargo mutants` on the issue diff (`101 tested: 82 caught, 19 unviable, 0 missed, 0 timeout`) using `CARGO_TARGET_DIR=target-fast-3300-mutants cargo mutants --in-place --in-diff /tmp/issue3300-current.diff -p tau-coding-agent -f crates/tau-coding-agent/src/live_rl_runtime.rs --baseline skip --timeout 180 -- --test-threads=1 live_rl_runtime::tests::`.
