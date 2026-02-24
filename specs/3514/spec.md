# Spec: Issue #3514 - M314 deterministic dashboard operator workflow depth gate

Status: Implemented

## Problem Statement
Dashboard live mutation depth is covered by M308, but operators still lack one
deterministic gate focused on higher-order ops workflow routes:
chat/session/lineage/memory-graph/tools route contracts in a single auditable
verification report.

## Scope
In scope:
- Add `scripts/verify/m314-dashboard-operator-workflow-depth.sh`.
- Add `scripts/verify/test-m314-dashboard-operator-workflow-depth.sh`.
- Emit deterministic JSON report artifact with required-step inventory.
- Map dashboard ops workflow contracts to executable selectors.
- Update README links with M314 verification entrypoint.

Out of scope:
- New dashboard feature implementation.
- New API schema/wire-format changes.
- External provider/network-dependent behavior changes.

## Acceptance Criteria
### AC-1 Deterministic M314 script emits dashboard workflow-depth report
Given local execution,
when `scripts/verify/m314-dashboard-operator-workflow-depth.sh` runs,
then it executes required selectors and writes a deterministic report artifact
with per-step status.

### AC-2 Dashboard operator workflow contracts are explicitly mapped/executable
Given M314 required-step inventory,
when script selectors execute,
then coverage includes ops chat selector/new-session flow, sessions lineage and
reset flows, memory-graph detail contracts, tools inventory contracts, and
last-action detail workflow.

### AC-3 Contract test enforces fail-closed required-step/report consistency
Given `scripts/verify/test-m314-dashboard-operator-workflow-depth.sh`,
when pass/fail/tamper paths run,
then missing required-step IDs or invalid report consistency fails closed.

### AC-4 README references M314 dashboard workflow-depth gate
Given README dashboard capability gap entries,
when reviewed,
then M314 script is included as an execution entrypoint.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M314 script | run | report artifact emitted with expected schema |
| C-02 | AC-2 | Functional | required-step map | execute script | ops workflow selectors run |
| C-03 | AC-2 | Integration | multi-route ops contracts | run script | deterministic workflow contracts pass |
| C-04 | AC-3 | Conformance/Regression | M314 script contract test | pass/fail/tamper execution | fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README links | inspect README | M314 link present |

## Success Metrics / Observable Signals
- One command verifies dashboard operator workflow depth with JSON output.
- Required-step checks detect silent coverage drift.
- README references the dashboard workflow-depth verification gate.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m314-dashboard-operator-workflow-depth.sh` passed and emitted `artifacts/dashboard-operator-workflow-depth/verification-report.json` (`suite_id=m314_dashboard_operator_workflow_depth`). |
| AC-2 | ✅ | Required M314 step mapping covers ops chat selector/new-session flow, sessions lineage/reset workflows, memory-graph detail contracts, tools inventory contracts, and last-action row contracts. |
| AC-3 | ✅ | `bash scripts/verify/test-m314-dashboard-operator-workflow-depth.sh` passed pass/fail/tamper contract paths and verify-only fail-closed required-step checks. |
| AC-4 | ✅ | `README.md` dashboard capability boundary notes and dashboard gap row include `scripts/verify/m314-dashboard-operator-workflow-depth.sh`. |
