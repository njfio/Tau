# Spec: Issue #3502 - M311 deterministic TUI operator workflow depth gate

Status: Implemented

## Problem Statement
TUI shell and shell-live contracts are covered by tests, but operators still
lack one deterministic gate focused on TUI workflow depth that aggregates panel
conformance, shell-live watch behavior, and artifact diagnostic fail-closed
contracts into one auditable report.

## Scope
In scope:
- Add `scripts/verify/m311-tui-operator-workflow-depth.sh`.
- Add `scripts/verify/test-m311-tui-operator-workflow-depth.sh`.
- Emit deterministic JSON report artifact with required-step inventory.
- Map TUI shell/shell-live contracts to executable selectors.
- Update README links with M311 verification entrypoint.

Out of scope:
- New TUI feature implementation.
- Dashboard or auth protocol behavior changes.
- New terminal rendering dependencies.

## Acceptance Criteria
### AC-1 Deterministic M311 script emits TUI workflow report
Given local execution,
when `scripts/verify/m311-tui-operator-workflow-depth.sh` runs,
then it executes required selectors and writes a deterministic report artifact
with per-step status.

### AC-2 TUI workflow contracts are explicitly mapped and executable
Given M311 required-step inventory,
when script selectors execute,
then coverage includes shell panel conformance, shell-live watch marker/help
contracts, and malformed/missing artifact diagnostic behavior.

### AC-3 Contract test enforces fail-closed required-step/report consistency
Given `scripts/verify/test-m311-tui-operator-workflow-depth.sh`,
when pass/fail/tamper paths run,
then missing required-step IDs or invalid report consistency fails closed.

### AC-4 README references M311 TUI workflow gate
Given README TUI capability gap entries,
when reviewed,
then M311 script is included as execution entrypoint.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M311 script | run | report artifact emitted with expected schema |
| C-02 | AC-2 | Functional | required-step map | execute script | TUI shell/shell-live/diagnostic selectors run |
| C-03 | AC-2 | Integration | multi-surface TUI selectors | run script | deterministic workflow contracts pass |
| C-04 | AC-3 | Conformance/Regression | M311 script contract test | pass/fail/tamper execution | fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README links | inspect README | M311 link present |

## Success Metrics / Observable Signals
- One command verifies TUI workflow depth with JSON output.
- Required-step checks detect silent coverage drift.
- README references the TUI workflow verification gate.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m311-tui-operator-workflow-depth.sh` passed and emitted `artifacts/tui-operator-workflow-depth/verification-report.json` (`suite_id=m311_tui_operator_workflow_depth`). |
| AC-2 | ✅ | Required M311 step mapping covers shell panel conformance, shell-live watch parse/help/marker contracts, and malformed/missing artifact diagnostics. |
| AC-3 | ✅ | `bash scripts/verify/test-m311-tui-operator-workflow-depth.sh` passed pass/fail/tamper contract paths and verify-only fail-closed required-step checks. |
| AC-4 | ✅ | `README.md` TUI capability boundary notes and TUI gap row include `scripts/verify/m311-tui-operator-workflow-depth.sh`. |
