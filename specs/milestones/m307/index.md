# M307 - C5 multi-channel E2E orchestration closure wave

Status: Active

## Context
M307 closes the remaining C5 multi-channel orchestration depth gap by shipping
an explicit deterministic verification gate that maps C5-01..C5-08 conformance
cases to executable selectors across `tau-multi-channel` and `tau-gateway`.

Primary sources:
- `specs/tau-e2e-testing-prd.md` (Scenario Group 5)
- `crates/tau-multi-channel/src/multi_channel_live_connectors.rs`
- `crates/tau-multi-channel/src/multi_channel_runtime/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Issue Hierarchy
- Epic: #3484
- Story: #3485
- Task: #3486

## Scope
- Add deterministic C5 verification script and report contract.
- Add script contract test with fail-closed required-step validation.
- Map C5-01..C5-08 to executable selectors.
- Update README execution links for discoverability.

## Exit Criteria
- `specs/3486/spec.md` is `Implemented` with AC->test evidence.
- M307 script report includes all required C5 step IDs exactly once.
- Contract test fails closed on missing selectors and passes on complete report.
- README includes M307 verification entrypoint.
