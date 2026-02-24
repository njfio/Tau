# Spec: Issue #3522 - M316 deterministic true RL operations drill-depth gate

Status: Implemented

## Problem Statement
M310 covers true RL policy operations depth, but there is no single
deterministic gate that fail-closes on missing operational-safety drill
contracts across M24 operational safety proof, resume-after-crash playbook,
benchmark significance/safety checks, and rollback checklist evidence.

## Scope
In scope:
- Add `scripts/verify/m316-rl-operations-drill-depth.sh`.
- Add `scripts/verify/test-m316-rl-operations-drill-depth.sh`.
- Emit deterministic JSON report artifact with required-step inventory.
- Aggregate M24 operational-safety and crash-recovery drill contracts.
- Update README true RL gap entry with M316 verification entrypoint.

Out of scope:
- New RL algorithm implementation changes.
- Live provider dependency additions.
- Non-verification protocol/schema changes.

## Acceptance Criteria
### AC-1 Deterministic M316 script emits true RL operations drill report
Given local execution,
when `scripts/verify/m316-rl-operations-drill-depth.sh` runs,
then it executes required selectors and writes a deterministic report artifact
with per-step status.

### AC-2 True RL operations drill contracts are explicitly mapped/executable
Given M316 required-step inventory,
when script selectors execute,
then coverage includes M310 policy-ops depth contract plus M24 operational
safety proof, resume-after-crash playbook validation, benchmark proof/significance/
safety-regression checks, and rollback checklist contracts.

### AC-3 Contract test enforces fail-closed required-step/report consistency
Given `scripts/verify/test-m316-rl-operations-drill-depth.sh`,
when pass/fail/tamper paths run,
then missing required-step IDs or invalid report consistency fails closed.

### AC-4 README references M316 true RL drill-depth gate
Given README true RL capability gap entry,
when reviewed,
then M316 script is included as an execution entrypoint.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M316 script | run | report artifact emitted with expected schema |
| C-02 | AC-2 | Functional | required-step map | execute script | true RL drill selectors run |
| C-03 | AC-2 | Integration | M24+M310 drill contracts | run script | deterministic contracts pass |
| C-04 | AC-3 | Conformance/Regression | M316 script contract test | pass/fail/tamper execution | fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README links | inspect README | M316 link present |

## Success Metrics / Observable Signals
- One command verifies true RL operations drill depth with JSON output.
- Required-step checks detect silent drill coverage drift.
- README references the M316 true RL drill-depth verification gate.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast TAU_M316_CARGO_TARGET_DIR=target-fast bash scripts/verify/m316-rl-operations-drill-depth.sh` passed and emitted `artifacts/rl-operations-drill-depth/verification-report.json` (`suite_id=m316_rl_operations_drill_depth`). |
| AC-2 | ✅ | Required M316 step mapping covers M310 policy-ops depth plus M24 operational safety proof, resume-after-crash playbook validation, live benchmark proof/significance/safety-regression checks, rollback checklist contract, and training runbook section checks. |
| AC-3 | ✅ | `bash scripts/verify/test-m316-rl-operations-drill-depth.sh` passed pass/fail/tamper contract paths and verify-only fail-closed required-step checks. |
| AC-4 | ✅ | `README.md` true RL capability boundary notes and true RL gap row include `scripts/verify/m316-rl-operations-drill-depth.sh`. |
