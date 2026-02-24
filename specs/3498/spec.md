# Spec: Issue #3498 - M310 deterministic RL policy-operations depth gate

Status: Implemented

## Problem Statement
True-RL productionization is integrated but still lacks one deterministic gate
that focuses specifically on RL policy-operations depth: promotion/rollback
gating, significance checks, benchmark proof contracts, and runtime promotion
audit logging in one auditable verification report.

## Scope
In scope:
- Add `scripts/verify/m310-rl-policy-ops-depth.sh`.
- Add `scripts/verify/test-m310-rl-policy-ops-depth.sh`.
- Emit deterministic JSON report artifact with required-step inventory.
- Map RL promotion/rollback/significance/runtime-audit contracts to executable
  selectors.
- Update README links with M310 verification entrypoint.

Out of scope:
- New RL algorithm changes.
- New runtime/benchmark wire formats.
- Live third-party/environment-dependent training runs.

## Acceptance Criteria
### AC-1 Deterministic M310 script emits RL policy-operations report
Given local execution,
when `scripts/verify/m310-rl-policy-ops-depth.sh` runs,
then it executes required selectors and writes a deterministic report artifact
with per-step status.

### AC-2 RL policy-operations contracts are explicitly mapped and executable
Given M310 required-step inventory,
when script selectors execute,
then coverage includes RL promotion/rollback gates, significance checks,
benchmark proof contracts, and runtime promotion audit-log regression.

### AC-3 Contract test enforces fail-closed required-step/report consistency
Given `scripts/verify/test-m310-rl-policy-ops-depth.sh`,
when pass/fail/tamper paths run,
then missing required-step IDs or invalid report consistency fails closed.

### AC-4 README references M310 RL policy-operations gate
Given README RL capability gap entries,
when reviewed,
then M310 script is included as an execution entrypoint.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M310 script | run | report artifact emitted with expected schema |
| C-02 | AC-2 | Functional | required-step map | execute script | RL promotion/rollback/significance/runtime-audit selectors run |
| C-03 | AC-2 | Integration | cross-crate RL selectors | run script | deterministic policy-ops contracts pass |
| C-04 | AC-3 | Conformance/Regression | M310 script contract test | pass/fail/tamper execution | fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README links | inspect README | M310 link present |

## Success Metrics / Observable Signals
- One command verifies RL policy-operations depth with JSON output.
- Required-step checks detect silent coverage drift.
- README references the RL policy-operations verification gate.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m310-rl-policy-ops-depth.sh` passed and emitted `artifacts/rl-policy-ops-depth/verification-report.json` (`suite_id=m310_rl_policy_ops_depth`). |
| AC-2 | ✅ | Required M310 step mapping covers RL promotion/rollback gates, significance checks, benchmark proof contracts, and runtime promotion-audit regression selectors. |
| AC-3 | ✅ | `bash scripts/verify/test-m310-rl-policy-ops-depth.sh` passed pass/fail/tamper contract paths and verify-only fail-closed required-step checks. |
| AC-4 | ✅ | `README.md` true-RL capability notes and true-RL gap row include `scripts/verify/m310-rl-policy-ops-depth.sh`. |
