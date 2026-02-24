# Spec: Issue #3527 - M317 deterministic TUI scenario-expansion depth gate

Status: Implemented

## Problem Statement
M311 covers core operator shell and shell-live workflow depth, but there is no
single deterministic gate that fail-closes on broader TUI scenario expansion
contracts across demo binary behavior, parser edge paths, and shell-live
argument error paths.

## Scope
In scope:
- Add `scripts/verify/m317-tui-scenario-expansion-depth.sh`.
- Add `scripts/verify/test-m317-tui-scenario-expansion-depth.sh`.
- Emit deterministic JSON report artifact with required-step inventory.
- Aggregate M311 + additional demo/parser/shell-live scenario contracts.
- Update README TUI gap entry with M317 verification entrypoint.

Out of scope:
- New TUI UI feature implementation.
- Non-verification protocol/schema changes.
- Web dashboard workflow changes.

## Acceptance Criteria
### AC-1 Deterministic M317 script emits TUI scenario-expansion report
Given local execution,
when `scripts/verify/m317-tui-scenario-expansion-depth.sh` runs,
then it executes required selectors and writes a deterministic report artifact
with per-step status.

### AC-2 TUI scenario-expansion contracts are explicitly mapped/executable
Given M317 required-step inventory,
when script selectors execute,
then coverage includes M311 workflow-depth contract plus demo-mode single/multi
frame behavior, demo invalid-frame rejection, parser shell/shell-live edge paths,
and shell-live watch argument contracts.

### AC-3 Contract test enforces fail-closed required-step/report consistency
Given `scripts/verify/test-m317-tui-scenario-expansion-depth.sh`,
when pass/fail/tamper paths run,
then missing required-step IDs or invalid report consistency fails closed.

### AC-4 README references M317 TUI scenario-expansion gate
Given README TUI capability gap entry,
when reviewed,
then M317 script is included as an execution entrypoint.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M317 script | run | report artifact emitted with expected schema |
| C-02 | AC-2 | Functional | required-step map | execute script | TUI scenario-expansion selectors run |
| C-03 | AC-2 | Integration | M311 + demo/parser/shell-live contracts | run script | deterministic contracts pass |
| C-04 | AC-3 | Conformance/Regression | M317 script contract test | pass/fail/tamper execution | fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README links | inspect README | M317 link present |

## Success Metrics / Observable Signals
- One command verifies TUI scenario-expansion depth with JSON output.
- Required-step checks detect silent TUI coverage drift.
- README references the M317 TUI scenario-expansion verification gate.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast TAU_M317_CARGO_TARGET_DIR=target-fast bash scripts/verify/m317-tui-scenario-expansion-depth.sh` passed and emitted `artifacts/tui-scenario-expansion-depth/verification-report.json` (`suite_id=m317_tui_scenario_expansion_depth`). |
| AC-2 | ✅ | Required M317 step mapping covers M311 workflow depth plus TUI demo single/multi-frame behavior, invalid-frame rejection, parser shell/shell-live edge paths, shell-live watch argument contracts, and docs contract checks. |
| AC-3 | ✅ | `bash scripts/verify/test-m317-tui-scenario-expansion-depth.sh` passed pass/fail/tamper contract paths and verify-only fail-closed required-step checks. |
| AC-4 | ✅ | `README.md` TUI capability boundary notes and TUI gap row include `scripts/verify/m317-tui-scenario-expansion-depth.sh`. |
