# Plan: Issue #3288

## Approach
1. Add `reward_inference.rs` to `tau-algorithm` with:
   - `RewardInferenceInput`
   - `RewardInferenceOutput`
   - `RewardInference` trait
   - `TraceBasedRewardInference` implementation
2. Export inference types from `tau-algorithm::lib`.
3. Add RED algorithm tests for deterministic output and hard-gate behavior.
4. Replace runtime-local reward breakdown implementation in `live_rl_runtime.rs` with algorithm inference call.
5. Re-run existing runtime conformance tests to confirm no regression.

## Affected Modules
- `crates/tau-algorithm/src/lib.rs`
- `crates/tau-algorithm/src/reward_inference.rs` (new)
- `crates/tau-coding-agent/src/live_rl_runtime.rs`

## Risks and Mitigations
- Risk: scoring drift after moving logic.
  Mitigation: preserve formula parity and assert expected outputs in existing runtime tests.
- Risk: public API churn in `tau-algorithm`.
  Mitigation: keep contract minimal and focused on immutable input/output structs.

## Interfaces / Contracts
New `tau-algorithm` contract:
- `RewardInference` trait (`infer(&RewardInferenceInput) -> RewardInferenceOutput`)
- `TraceBasedRewardInference` default implementation.

## ADR
Not required (no architectural boundary/dependency/protocol change).
