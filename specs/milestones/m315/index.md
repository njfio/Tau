# M315 - E2E operator-route scenario depth verification wave

Status: Active

## Context
M315 deepens E2E coverage by adding one deterministic gate that aggregates
operator-route scenario contracts across memory/tools/channels/config/training
ops routes into a single auditable verification report.

Primary sources:
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/tau-e2e-testing-prd.md`
- `specs/tau-ops-dashboard-prd.md`

## Issue Hierarchy
- Epic: #3516
- Story: #3517
- Task: #3518

## Scope
- Add deterministic M315 operator-route E2E verification script and report.
- Add script contract test with fail-closed required-step checks.
- Map operator-route scenario contracts to executable selectors.
- Update README links with M315 verification entrypoint.

## Exit Criteria
- `specs/3518/spec.md` is `Implemented` with AC evidence.
- M315 script report includes all required operator-route step IDs.
- Contract test fails closed on missing required-step IDs.
- README includes M315 verification entrypoint.
