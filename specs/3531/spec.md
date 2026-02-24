# Spec: Issue #3531 - M318 deterministic dashboard command-center depth gate

Status: Implemented

## Problem Statement
M308 covers dashboard live mutation depth and M314 covers operator workflow
depth, but there is no single deterministic gate that fail-closes on broader
dashboard command-center contracts across timeline/alert/control markers and
live stream matrix behavior.

## Scope
In scope:
- Add `scripts/verify/m318-dashboard-command-center-depth.sh`.
- Add `scripts/verify/test-m318-dashboard-command-center-depth.sh`.
- Emit deterministic JSON report artifact with required-step inventory.
- Aggregate M308 + M314 + command-center/timeline/alert/control/live-stream contracts.
- Update README dashboard gap entry with M318 verification entrypoint.

Out of scope:
- New dashboard UI feature implementation.
- Non-verification protocol/schema changes.
- Multi-channel/auth workflow changes.

## Acceptance Criteria
### AC-1 Deterministic M318 script emits dashboard command-center report
Given local execution,
when `scripts/verify/m318-dashboard-command-center-depth.sh` runs,
then it executes required selectors and writes a deterministic report artifact
with per-step status.

### AC-2 Dashboard command-center contracts are explicitly mapped/executable
Given M318 required-step inventory,
when script selectors execute,
then coverage includes M308 and M314 contract gates plus command-center markers,
timeline markers/range defaults, alert-feed markers, connector health markers,
control markers/confirmation payload, control-action fail paths, and live stream
matrix contracts.

### AC-3 Contract test enforces fail-closed required-step/report consistency
Given `scripts/verify/test-m318-dashboard-command-center-depth.sh`,
when pass/fail/tamper paths run,
then missing required-step IDs or invalid report consistency fails closed.

### AC-4 README references M318 dashboard command-center depth gate
Given README dashboard capability gap entry,
when reviewed,
then M318 script is included as an execution entrypoint.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M318 script | run | report artifact emitted with expected schema |
| C-02 | AC-2 | Functional | required-step map | execute script | dashboard command-center selectors run |
| C-03 | AC-2 | Integration | M308 + M314 + command-center contracts | run script | deterministic contracts pass |
| C-04 | AC-3 | Conformance/Regression | M318 script contract test | pass/fail/tamper execution | fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README links | inspect README | M318 link present |

## Success Metrics / Observable Signals
- One command verifies dashboard command-center depth with JSON output.
- Required-step checks detect silent dashboard coverage drift.
- README references the M318 dashboard command-center verification gate.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast TAU_M318_CARGO_TARGET_DIR=target-fast bash scripts/verify/m318-dashboard-command-center-depth.sh` passed and emitted `artifacts/dashboard-command-center-depth/verification-report.json` (`suite_id=m318_dashboard_command_center_depth`). |
| AC-2 | ✅ | Required M318 step mapping covers M308 + M314 contract gates plus command-center/timeline/alert/control markers, control-action fail paths, live stream matrix contract, and dashboard runbook API surface checks. |
| AC-3 | ✅ | `bash scripts/verify/test-m318-dashboard-command-center-depth.sh` passed pass/fail/tamper contract paths and verify-only fail-closed required-step checks. |
| AC-4 | ✅ | `README.md` dashboard capability boundary notes and dashboard gap row include `scripts/verify/m318-dashboard-command-center-depth.sh`. |
