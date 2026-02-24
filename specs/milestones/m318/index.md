# M318 - Dashboard command-center depth verification wave

Status: Active

## Context
M318 deepens dashboard maturity verification by adding one deterministic gate
that aggregates existing M308/M314 dashboard depth contracts plus command-center,
timeline, alert-feed, control-marker, and dashboard live stream matrix contracts
into a single auditable report.

Primary sources:
- `scripts/verify/m308-dashboard-live-mutation-depth.sh`
- `scripts/verify/m314-dashboard-operator-workflow-depth.sh`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `docs/guides/dashboard-ops.md`

## Issue Hierarchy
- Epic: #3529
- Story: #3530
- Task: #3531

## Scope
- Add deterministic M318 dashboard command-center depth script and report.
- Add script contract test with fail-closed required-step checks.
- Map command-center/timeline/alert/control/live-stream contracts to selectors.
- Update README links with M318 verification entrypoint.

## Exit Criteria
- `specs/3531/spec.md` is `Implemented` with AC evidence.
- M318 report includes all required dashboard command-center depth step IDs.
- Contract test fails closed on missing required-step IDs.
- README dashboard gap entry includes M318 verification entrypoint.
