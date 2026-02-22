# Spec: Issue #3288 - add RewardInference trait and wire live RL runtime to TraceBasedRewardInference

Status: Implemented

## Problem Statement
Reward scoring for live RL is embedded in runtime-local logic, preventing reuse and independent evolution. A dedicated algorithm-layer inference contract is needed to compute deterministic reward breakdowns from trace-like inputs.

## Scope
In scope:
- Add a `RewardInference` trait and `TraceBasedRewardInference` implementation in `tau-algorithm`.
- Add algorithm conformance tests for deterministic component and composite scoring.
- Replace local live runtime reward breakdown logic with algorithm inference usage.

Out of scope:
- PPO/GAE optimizer changes.
- Training store schema updates.
- External wire/API contract changes.

## Acceptance Criteria
### AC-1 algorithm contract computes deterministic breakdown
Given identical inference inputs,
when `TraceBasedRewardInference` computes rewards,
then composite and component fields are deterministic and bounded.

### AC-2 safety hard-gate remains fail-closed
Given `safety_blocked = true`,
when inference executes,
then composite score is `-1.0` regardless of other positive signals.

### AC-3 live runtime consumes algorithm inference without behavior regression
Given a live run event flow,
when `live.agent.decision` span is emitted,
then existing reward breakdown attributes are still present and consistent with inference outputs.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Unit/Conformance | fixed trace input with no safety block | algorithm inference runs | deterministic component values and bounded composite |
| C-02 | AC-2 | Unit/Regression | safety-blocked trace input | algorithm inference runs | composite equals `-1.0` |
| C-03 | AC-3 | Functional/Conformance | live runtime completes a run | span persisted | reward + component keys exist and match expected values |

## Success Metrics / Observable Signals
- `cargo test -p tau-algorithm spec_c01_unit_trace_based_reward_inference_computes_components`
- `cargo test -p tau-algorithm spec_c02_regression_trace_based_reward_inference_safety_hard_gate`
- `cargo test -p tau-coding-agent spec_c05_unit_live_reward_breakdown_scores_deterministically`
- `cargo test -p tau-coding-agent spec_c06_functional_live_rollout_span_persists_reward_breakdown`
- `cargo fmt --check`
- `cargo clippy -p tau-algorithm --no-deps -- -D warnings`
- `cargo clippy -p tau-coding-agent --no-deps -- -D warnings`
