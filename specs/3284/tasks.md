# Tasks: Issue #3284 - add composite live reward evaluator and span breakdown attributes

- [ ] T1 (RED): add failing conformance tests for deterministic reward breakdown values and span attribute persistence.
- [ ] T2 (GREEN): implement `LiveRewardBreakdown` scoring and replace scalar reward computation.
- [ ] T3 (GREEN): persist per-dimension reward attributes in final decision span.
- [ ] T4 (VERIFY): run targeted tau-coding-agent tests for C-01/C-02/C-03.
- [ ] T5 (VERIFY): run `cargo fmt --check` and `cargo clippy -p tau-coding-agent --no-deps -- -D warnings`.
