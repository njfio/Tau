# Tasks: Issue #3292 - expand TraceBasedRewardInference with session completion and token efficiency

- [ ] T1 (RED): add failing unit tests for token-efficiency contribution and non-completed-session penalty.
- [ ] T2 (GREEN): implement signal expansion in `TraceBasedRewardInference` input/output and scoring logic.
- [ ] T3 (GREEN): persist new reward breakdown fields in live runtime span attributes.
- [ ] T4 (VERIFY): run targeted algorithm/runtime conformance tests.
- [ ] T5 (VERIFY): run fmt + clippy (`tau-algorithm`, `tau-coding-agent`) with `-D warnings`.
