# Plan: Issue #3292

## Approach
1. Extend `RewardInferenceInput` with `session_completed`.
2. Extend `RewardInferenceOutput` with:
   - `session_completion`
   - `token_efficiency`
3. Update `TraceBasedRewardInference` formula:
   - Add completion penalty when session not completed.
   - Add bounded token-efficiency contribution from output/input ratio.
4. Add RED tests in `tau-algorithm` for new signal behavior.
5. Wire live runtime span emission with new attributes and assert via existing functional test.

## Affected Modules
- `crates/tau-algorithm/src/reward_inference.rs`
- `crates/tau-coding-agent/src/live_rl_runtime.rs`

## Risks and Mitigations
- Risk: reward drift breaks prior expectations.
  Mitigation: preserve previous component ranges and update deterministic tests for explicit new terms.
- Risk: runtime attribute contract drift.
  Mitigation: functional conformance assertion for new attribute keys.

## Interfaces / Contracts
- Backward-compatible struct extension in algorithm contract.
- Runtime span attribute additions only.

## ADR
Not required.
