# Plan: Issue #3402 - Final C5/K13/R8 conformance closure

## Approach
1. Reuse deterministic existing coverage where scenario intent already exists:
   - Multi-channel secure envelope accept/reject and media understanding tests for `C5-03`, `C5-04`, `C5-08`.
   - Auth rotate-key lifecycle tests for `K13-06`.
   - Live RL rollout creation/optimizer interval and reward inference token/session tests for `R8-01`, `R8-02`, `R8-03`, `R8-05`, `R8-08`.
2. Add focused missing assertions:
   - Multi-channel ingress routing test that explicitly exercises Telegram + Discord inbound persistence/routing for `C5-01`, `C5-02`, `C5-07`.
   - Reward inference reliability penalty test asserting tool errors reduce reliability for `R8-04`.
3. Update conformance artifacts:
   - mark all in-scope rows as `Covered` in `specs/3386/conformance-matrix.md`.
   - add issue-local mapping in `specs/3402/conformance-matrix.md`.

## Affected Modules
- `crates/tau-multi-channel/src/multi_channel_runtime/tests.rs`
- `crates/tau-algorithm/src/reward_inference.rs`
- `specs/3386/conformance-matrix.md`
- `specs/3402/conformance-matrix.md`
- `specs/milestones/m291/index.md`

## Risks / Mitigations
- Risk: Scenario language references webhook/HMAC semantics while runtime uses signed envelope enforcement.
  - Mitigation: map to deterministic inbound signature validation/rejection coverage with explicit notes in matrix rows.
- Risk: Over-mapping internal tests to scenario intent could weaken traceability quality.
  - Mitigation: add focused missing tests for the two weakest rows (`C5-01/02/07`, `R8-04`) to keep mappings concrete.

## Interfaces / Contracts
- No new runtime/public API contracts.
- Conformance closure through deterministic test coverage and spec/matrix traceability updates only.

## ADR
- Not required (no dependency, protocol, or architecture changes).
