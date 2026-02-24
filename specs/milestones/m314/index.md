# M314 - Dashboard operator workflow depth verification wave

Status: Active

## Context
M314 deepens dashboard verification by adding one deterministic gate that
aggregates ops chat/session/lineage/memory-graph/tools workflow contracts into
a single auditable report.

Primary sources:
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/tau-ops-dashboard-prd.md`
- `docs/guides/dashboard-ops.md`

## Issue Hierarchy
- Epic: #3512
- Story: #3513
- Task: #3514

## Scope
- Add deterministic M314 dashboard workflow verification script and report.
- Add script contract test with fail-closed required-step checks.
- Map ops route workflow coverage to executable selectors.
- Update README links with M314 verification entrypoint.

## Exit Criteria
- `specs/3514/spec.md` is `Implemented` with AC evidence.
- M314 script report includes all required dashboard workflow step IDs.
- Contract test fails closed on missing required-step IDs.
- README includes M314 verification entrypoint.
