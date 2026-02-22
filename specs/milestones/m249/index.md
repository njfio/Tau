# M249 - trace-based reward inference contract

Status: In Progress

## Context
Live RL scoring currently lives in runtime-local logic. Review #35 recommends moving scoring behind an algorithm-layer contract to make reward inference reusable and evolvable.

## Scope
- Add `RewardInference` contract to `tau-algorithm`.
- Implement deterministic `TraceBasedRewardInference`.
- Wire live RL runtime to consume algorithm-layer inference while preserving reward span attributes.

## Linked Issues
- Epic: #3286
- Story: #3287
- Task: #3288

## Success Signals
- `cargo test -p tau-algorithm spec_c01_unit_trace_based_reward_inference_computes_components`
- `cargo test -p tau-algorithm spec_c02_regression_trace_based_reward_inference_safety_hard_gate`
- `cargo test -p tau-coding-agent spec_c05_unit_live_reward_breakdown_scores_deterministically`
- `cargo test -p tau-coding-agent spec_c06_functional_live_rollout_span_persists_reward_breakdown`
- `cargo fmt --check`
- `cargo clippy -p tau-algorithm --no-deps -- -D warnings`
- `cargo clippy -p tau-coding-agent --no-deps -- -D warnings`
