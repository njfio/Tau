# Spec: Issue #3292 - expand TraceBasedRewardInference with session completion and token efficiency

Status: Reviewed

## Problem Statement
The current trace-based inference contract does not directly encode session completion and token efficiency despite these being key intrinsic signals from Review #35.

## Scope
In scope:
- Extend `RewardInferenceInput` with `session_completed`.
- Add deterministic token-efficiency component derived from input/output character ratio.
- Persist token-efficiency and session-completion components in live runtime span attributes.

Out of scope:
- Changes to PPO/GAE.
- External API/wire contract changes.
- Storage schema changes.

## Acceptance Criteria
### AC-1 token-efficiency signal contributes deterministically
Given fixed input/output character telemetry,
when inference executes,
then `token_efficiency` and composite score are deterministic and bounded.

### AC-2 incomplete session applies deterministic penalty
Given `session_completed = false`,
when inference executes,
then output includes completion penalty contribution while preserving safety hard-gate behavior.

### AC-3 live runtime persists expanded reward breakdown
Given final decision span emission,
when run completes,
then span includes reward attributes for token efficiency and session completion signals.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Unit/Conformance | non-blocked run with ratio telemetry | inference executes | deterministic `token_efficiency` and bounded composite |
| C-02 | AC-2 | Unit/Regression | non-completed run input | inference executes | session completion penalty applied |
| C-03 | AC-3 | Functional/Conformance | runtime emits `live.agent.decision` span | span persisted | includes `reward_token_efficiency` and `reward_session_completion` |

## Success Metrics / Observable Signals
- `cargo test -p tau-algorithm spec_c03_unit_trace_based_reward_inference_token_efficiency_signal`
- `cargo test -p tau-algorithm spec_c04_regression_trace_based_reward_inference_session_not_completed_penalty`
- `cargo test -p tau-coding-agent spec_c06_functional_live_rollout_span_persists_reward_breakdown`
- `cargo fmt --check`
- `cargo clippy -p tau-algorithm --no-deps -- -D warnings`
- `cargo clippy -p tau-coding-agent --no-deps -- -D warnings`
