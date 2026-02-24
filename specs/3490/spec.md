# Spec: Issue #3490 - M308 deterministic dashboard live mutation depth gate

Status: Implemented

## Problem Statement
Dashboard live mutation behavior is covered by distributed tests but lacks a
single deterministic operator-facing verification gate that aggregates the key
contracts (status endpoints, control actions, stream reconnect semantics, and
ops-shell markers) into one auditable report.

## Scope
In scope:
- Add `scripts/verify/m308-dashboard-live-mutation-depth.sh`.
- Add `scripts/verify/test-m308-dashboard-live-mutation-depth.sh`.
- Emit deterministic JSON report with required dashboard verification steps.
- Map dashboard live mutation contracts to deterministic selectors.
- Update README links with M308 gate entrypoint.

Out of scope:
- New dashboard endpoint or payload schema changes.
- New frontend feature implementation.
- External network/provider-dependent dashboard validation.

## Acceptance Criteria
### AC-1 Deterministic M308 script emits dashboard verification report
Given local repository execution,
when `scripts/verify/m308-dashboard-live-mutation-depth.sh` runs,
then it executes required dashboard selectors and emits a deterministic JSON
report with per-step pass/fail status.

### AC-2 Dashboard live mutation contracts are explicitly mapped and executable
Given M308 required-step inventory,
when script steps are inspected and executed,
then coverage includes dashboard status endpoints, control-action mutation
flows, SSE reset/snapshot reconnect semantics, and ops-shell markers.

### AC-3 Contract test fails closed on required-step or report inconsistencies
Given `scripts/verify/test-m308-dashboard-live-mutation-depth.sh`,
when pass/fail/tamper paths run,
then missing required step IDs or invalid report consistency fails closed.

### AC-4 README references the M308 dashboard verification gate
Given README current-gap/capability references,
when reviewed,
then M308 script is linked as dashboard maturity verification entrypoint.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M308 entrypoint | run script | report artifact created with expected schema |
| C-02 | AC-2 | Functional | required dashboard step map | execute script | dashboard status/action/stream/contracts selectors run |
| C-03 | AC-2 | Integration | gateway dashboard selectors | run script | live mutation pathways pass deterministically |
| C-04 | AC-3 | Regression/Conformance | script contract test | run pass/fail/tamper | fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README docs links | inspect README | M308 link present |

## Success Metrics / Observable Signals
- One command verifies dashboard live mutation depth with JSON output.
- Required-step checks prevent silent contract coverage drift.
- README links point operators to the dashboard depth gate command.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m308-dashboard-live-mutation-depth.sh` passed and emitted `artifacts/dashboard-live-mutation-depth/verification-report.json` (`suite_id=m308_dashboard_live_mutation_depth`). |
| AC-2 | ✅ | Required M308 step mapping covers dashboard status endpoints, action mutation persistence, SSE reconnect reset/snapshot semantics, ops-shell control-action mutation path, and unauthorized fail-closed behavior. |
| AC-3 | ✅ | `bash scripts/verify/test-m308-dashboard-live-mutation-depth.sh` passed pass/fail/tamper contract paths and verify-only fail-closed marker checks. |
| AC-4 | ✅ | `README.md` dashboard maturity gap row now includes `scripts/verify/m308-dashboard-live-mutation-depth.sh`. |
