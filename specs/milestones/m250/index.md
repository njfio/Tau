# M250 - reward inference signal expansion

Status: In Progress

## Context
`TraceBasedRewardInference` exists, but Phase-1 autonomy work still lacks explicit session-outcome and token-efficiency signal usage.

## Scope
- Expand inference input contract with session completion.
- Add deterministic token-efficiency component from input/output character telemetry.
- Surface new component in live runtime decision span attributes.

## Linked Issues
- Epic: #3290
- Story: #3291
- Task: #3292

## Success Signals
- `cargo test -p tau-algorithm spec_c03_unit_trace_based_reward_inference_token_efficiency_signal`
- `cargo test -p tau-algorithm spec_c04_regression_trace_based_reward_inference_session_not_completed_penalty`
- `cargo test -p tau-coding-agent spec_c06_functional_live_rollout_span_persists_reward_breakdown`
- `cargo fmt --check`
- `cargo clippy -p tau-algorithm --no-deps -- -D warnings`
- `cargo clippy -p tau-coding-agent --no-deps -- -D warnings`
