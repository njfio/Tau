# Tasks: Issue #3288 - add RewardInference trait and wire live RL runtime to TraceBasedRewardInference

- [x] T1 (RED): add failing tau-algorithm conformance tests for deterministic components and safety hard-gate behavior.
- [x] T2 (GREEN): implement `RewardInference` contract and `TraceBasedRewardInference` in `tau-algorithm`.
- [x] T3 (GREEN): switch `live_rl_runtime` to use `TraceBasedRewardInference` outputs.
- [x] T4 (VERIFY): run tau-algorithm + tau-coding-agent conformance/regression tests.
- [x] T5 (VERIFY): run `cargo fmt --check` and clippy (`tau-algorithm`, `tau-coding-agent`) with `-D warnings`.
