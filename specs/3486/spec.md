# Spec: Issue #3486 - M307 deterministic C5 multi-channel orchestration depth gate

Status: Implemented

## Problem Statement
Scenario Group 5 (C5 multi-channel orchestration) has broad underlying test
coverage but no single deterministic operator-facing gate proving C5-01..C5-08
coverage in one auditable report. Without that gate, C5 remains operationally
partial despite distributed test assets.

## Scope
In scope:
- Add `scripts/verify/m307-multi-channel-orchestration-depth.sh`.
- Add `scripts/verify/test-m307-multi-channel-orchestration-depth.sh`.
- Emit deterministic JSON verification report with required C5 step inventory.
- Map C5-01..C5-08 to executable selectors across `tau-multi-channel` and
  `tau-gateway`.
- Update README links to include the M307 verification entrypoint.

Out of scope:
- New multi-channel runtime features/protocol changes.
- Live external provider calls.
- Reworking C5 PRD definitions.

## Acceptance Criteria
### AC-1 Deterministic M307 verification script emits conformance report
Given local workspace execution,
when `scripts/verify/m307-multi-channel-orchestration-depth.sh` runs,
then it executes all required C5 selector steps and writes a deterministic JSON
report artifact with suite metadata and per-step pass/fail state.

### AC-2 C5-01..C5-08 coverage is explicitly mapped and executable
Given Scenario Group 5 conformance cases,
when script steps are inspected/executed,
then each C5 case is represented by at least one deterministic selector and the
report marks each mapped step.

### AC-3 Script contract test enforces fail-closed required-step inventory
Given `scripts/verify/test-m307-multi-channel-orchestration-depth.sh`,
when it runs pass/fail/tamper paths,
then missing required step IDs or inconsistent report status fail closed.

### AC-4 README references the M307 C5 verification gate
Given README verification links,
when reviewed,
then the M307 script is discoverable under integration/auth/dashboard/TUI gap
execution references.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M307 script entrypoint | run script | JSON report written with valid schema/suite/steps |
| C-02 | AC-2 | Functional | C5 inventory map | run script selectors | C5-01..C5-08 coverage markers are present |
| C-03 | AC-2 | Integration | multi-crate selector set | execute script | mapped gateway + multi-channel selectors pass deterministically |
| C-04 | AC-3 | Conformance/Regression | script contract test | run pass/fail/tamper cases | required-step and fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README links | inspect README | M307 gate link is present |

## Success Metrics / Observable Signals
- One command verifies C5 depth with deterministic JSON output.
- Required-step enforcement prevents silent selector drift.
- README points operators to the C5 gate command.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m307-multi-channel-orchestration-depth.sh` passed and emitted `artifacts/multi-channel-orchestration-depth/verification-report.json` with `suite_id=m307_multi_channel_orchestration_depth`. |
| AC-2 | ✅ | M307 required steps explicitly map C5-01..C5-08 selectors (routing matrix, WhatsApp valid/invalid signature, lifecycle contracts, media-attachment handling). Report validation enforces exact-once required-step IDs. |
| AC-3 | ✅ | `bash scripts/verify/test-m307-multi-channel-orchestration-depth.sh` passed pass/fail/tamper contract paths; verify-only mode fails closed when a required step is removed. |
| AC-4 | ✅ | `README.md` now references `scripts/verify/m307-multi-channel-orchestration-depth.sh` in capability-boundary and current-gap execution sections. |
