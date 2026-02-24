# Spec: Issue #3518 - M315 deterministic E2E operator-route scenario depth gate

Status: Implemented

## Problem Statement
M313 covers E2E core lifecycle/session flows, but operator-route scenario depth
across memory/tools/channels/config/training routes still lacks one dedicated
deterministic gate with fail-closed required-step inventory checks.

## Scope
In scope:
- Add `scripts/verify/m315-e2e-operator-route-depth.sh`.
- Add `scripts/verify/test-m315-e2e-operator-route-depth.sh`.
- Emit deterministic JSON report artifact with required-step inventory.
- Map operator-route E2E scenario contracts to executable selectors.
- Update README links with M315 verification entrypoint.

Out of scope:
- Full completion of all PRD scenario groups.
- New dashboard feature behavior changes.
- New live-provider dependencies in required path.

## Acceptance Criteria
### AC-1 Deterministic M315 script emits operator-route E2E report
Given local execution,
when `scripts/verify/m315-e2e-operator-route-depth.sh` runs,
then it executes required selectors and writes a deterministic report artifact
with per-step status.

### AC-2 Operator-route E2E contracts are explicitly mapped and executable
Given M315 required-step inventory,
when script selectors execute,
then coverage includes ops memory workflows (search/filter/create/edit/delete/detail/graph),
ops tools routes, ops channels routes, and ops config/training/safety/diagnostics panels.

### AC-3 Contract test enforces fail-closed required-step/report consistency
Given `scripts/verify/test-m315-e2e-operator-route-depth.sh`,
when pass/fail/tamper paths run,
then missing required-step IDs or invalid report consistency fails closed.

### AC-4 README references M315 operator-route E2E gate
Given README E2E capability gap entries,
when reviewed,
then M315 script is included as an execution entrypoint.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M315 script | run | report artifact emitted with expected schema |
| C-02 | AC-2 | Functional | required-step map | execute script | operator-route selectors run |
| C-03 | AC-2 | Integration | multi-route operator scenario contracts | run script | deterministic route contracts pass |
| C-04 | AC-3 | Conformance/Regression | M315 script contract test | pass/fail/tamper execution | fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README links | inspect README | M315 link present |

## Success Metrics / Observable Signals
- One command verifies operator-route E2E scenario depth with JSON output.
- Required-step checks detect silent coverage drift.
- README references the operator-route E2E verification gate.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m315-e2e-operator-route-depth.sh` passed and emitted `artifacts/e2e-operator-route-depth/verification-report.json` (`suite_id=m315_e2e_operator_route_depth`). |
| AC-2 | ✅ | Required M315 step mapping covers ops memory workflows (search/filter/create/edit/delete/detail/graph), ops tools routes, ops channels routes, and ops config/training/safety/diagnostics panels. |
| AC-3 | ✅ | `bash scripts/verify/test-m315-e2e-operator-route-depth.sh` passed pass/fail/tamper contract paths and verify-only fail-closed required-step checks. |
| AC-4 | ✅ | `README.md` E2E capability boundary notes and E2E scenario-group gap row include `scripts/verify/m315-e2e-operator-route-depth.sh`. |
