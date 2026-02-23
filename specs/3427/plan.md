# Plan: Issue #3427 - RL fail-closed reason-code determinism

## Approach
1. Add RED tests for APO fail-closed paths where reason-code determinism is required.
2. Apply minimal runtime change to normalize unstable reason-code values.
3. Re-run targeted `live_rl_runtime` selectors plus related existing APO regression selectors.
4. Capture RED/GREEN/REGRESSION evidence in tasks artifact.

## Implementation Targets
- `crates/tau-coding-agent/src/live_rl_runtime.rs`
- `specs/milestones/m296/index.md`
- `specs/3427/spec.md`
- `specs/3427/plan.md`
- `specs/3427/tasks.md`

## Risks and Mitigations
- Risk: tightening reason codes may hide debugging context.
  - Mitigation: keep deterministic `reason_code` for contracts; retain detailed error context in tracing logs.
- Risk: changes may break existing APO tests expecting prior string format.
  - Mitigation: update/add tests explicitly around stable reason codes and rerun neighboring APO selectors.
- Risk: modifying high-churn RL file may trigger style/lint regressions.
  - Mitigation: keep patch localized to reason-code branches and new tests only.

## Interfaces / Contracts
- Internal runtime contract in `LiveApoReport.reason_code`.
- Existing live RL conformance test suite under `live_rl_runtime::tests::*`.
- No external wire/API contract expansion.

## ADR
Not required (no architecture/dependency/protocol change).
