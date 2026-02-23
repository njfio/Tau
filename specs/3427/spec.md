# Spec: Issue #3427 - True RL productionization fail-closed gating determinism

Status: Implemented

## Problem Statement
Live RL runtime includes APO significance gating and fail-closed skip behavior, but error-path `reason_code` values currently embed dynamic error text. That makes operator alerts, dashboard parsing, and regression assertions less deterministic than production gates require.

## Scope
In scope:
- Add conformance/regression tests for APO fail-closed error paths in live RL runtime.
- Harden APO skip `reason_code` values to deterministic machine-stable codes.
- Verify optimizer reporting keeps APO failures non-adoptive and observable.

Out of scope:
- New RL algorithms or optimizer math changes.
- New dashboard API/wire contract changes.
- Provider/runtime architecture redesign.

## Acceptance Criteria
### AC-1 APO missing-runtime and insufficient-sample gates stay fail-closed
Given live RL runtime APO preconditions are not met,
when APO update is attempted,
then APO execution is skipped with deterministic reason codes and no prompt adoption.

### AC-2 APO algorithm runtime failures emit deterministic reason codes
Given APO algorithm execution fails,
when the runtime reports APO outcome,
then `reason_code` is the stable value `apo_run_failed` and adoption remains false.

### AC-3 APO significance engine failures emit deterministic reason codes
Given significance comparison fails (for unsupported alpha/input),
when APO outcome is reported,
then `reason_code` is the stable value `apo_significance_failed` and adoption remains false.

### AC-4 Optimizer reports remain operationally parseable
Given APO skip/failure outcomes,
when optimizer report snapshots are read,
then report fields are deterministic and suitable for downstream alerting/contract assertions.

### AC-5 Spec-driven artifacts and verification evidence are complete
Given AGENTS process requirements,
when issue artifacts are reviewed,
then `spec.md`, `plan.md`, and `tasks.md` exist with AC/C-case/test mapping and RED/GREEN/REGRESSION evidence.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Regression | APO runtime disabled/missing | run APO update path | report `reason_code=apo_missing_runtime`, `executed=false`, `adopted=false` |
| C-02 | AC-1 | Regression | insufficient sample window | run APO update path | report `reason_code=apo_insufficient_samples`, no adoption |
| C-03 | AC-2 | Regression | algorithm client failure | run APO update path | report `reason_code=apo_run_failed`, no adoption |
| C-04 | AC-3 | Regression | unsupported significance alpha | run APO update path | report `reason_code=apo_significance_failed`, no adoption |
| C-05 | AC-4 | Functional/Conformance | optimizer snapshot includes APO report | read runtime snapshot | deterministic reason codes and report fields are present |
| C-06 | AC-5 | Conformance | issue artifact paths | verify files/sections | spec/plan/tasks exist and map to tests |

## Success Metrics / Observable Signals
- New RL fail-closed tests pass and lock deterministic reason-code behavior.
- Existing APO adoption/non-adoption tests remain green.
- No regression in existing `live_rl_runtime` conformance selectors.
