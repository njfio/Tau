# Spec: Issue #3510 - M313 deterministic E2E core scenario depth gate

Status: Implemented

## Problem Statement
Tau has integrated E2E slices across `tests/integration` and gateway tests, but
operators lack one deterministic gate that aggregates core scenario-depth
coverage into a single auditable report with fail-closed step inventory checks.

## Scope
In scope:
- Add `scripts/verify/m313-e2e-core-scenario-depth.sh`.
- Add `scripts/verify/test-m313-e2e-core-scenario-depth.sh`.
- Emit deterministic JSON report artifact with required-step inventory.
- Map integration-package and gateway E2E core selectors to executable steps.
- Update README links with M313 verification entrypoint.

Out of scope:
- Full implementation of all scenario groups in `specs/tau-e2e-testing-prd.md`.
- New gateway feature behavior changes.
- New live-provider network dependencies.

## Acceptance Criteria
### AC-1 Deterministic M313 script emits E2E core-depth report
Given local execution,
when `scripts/verify/m313-e2e-core-scenario-depth.sh` runs,
then it executes required selectors and writes a deterministic report artifact
with per-step status.

### AC-2 E2E core scenario contracts are explicitly mapped and executable
Given M313 required-step inventory,
when script selectors execute,
then coverage includes integration memory/tool scenarios, gateway lifecycle and
session flow harness tests, and gateway HTTP/WS/auth session core contracts.

### AC-3 Contract test enforces fail-closed required-step/report consistency
Given `scripts/verify/test-m313-e2e-core-scenario-depth.sh`,
when pass/fail/tamper paths run,
then missing required-step IDs or invalid report consistency fails closed.

### AC-4 README references M313 E2E core-depth gate
Given README integration/gap entries,
when reviewed,
then M313 script is included as an execution entrypoint.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M313 script | run | report artifact emitted with expected schema |
| C-02 | AC-2 | Functional | required-step map | execute script | integration + gateway E2E selectors run |
| C-03 | AC-2 | Integration | cross-package selector set | run script | deterministic core E2E contracts pass |
| C-04 | AC-3 | Conformance/Regression | M313 script contract test | pass/fail/tamper execution | fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README links | inspect README | M313 link present |

## Success Metrics / Observable Signals
- One command verifies E2E core scenario depth with JSON output.
- Required-step checks detect silent coverage drift.
- README references the E2E core-depth verification gate.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m313-e2e-core-scenario-depth.sh` passed and emitted `artifacts/e2e-core-scenario-depth/verification-report.json` (`suite_id=m313_e2e_core_scenario_depth`). |
| AC-2 | ✅ | Required M313 step mapping covers integration package scenarios (workspace run, memory roundtrip, isolation/channel-scope) and gateway lifecycle/session/http/ws/auth contracts. |
| AC-3 | ✅ | `bash scripts/verify/test-m313-e2e-core-scenario-depth.sh` passed pass/fail/tamper contract paths and verify-only fail-closed required-step checks. |
| AC-4 | ✅ | `README.md` capability boundaries and current-gaps table include `scripts/verify/m313-e2e-core-scenario-depth.sh`. |
