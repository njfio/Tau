# Plan: Issue #3531 - M318 deterministic dashboard command-center depth gate

Status: Implemented

## Approach
1. Add RED-first contract test for missing M318 script.
2. Implement M318 script with deterministic selector mapping and JSON report
   emission.
3. Enforce fail-closed report schema + required-step checks.
4. Update README dashboard gap links with M318 entrypoint.
5. Run RED->GREEN->REGRESSION verification and update spec evidence.

## Affected Modules
- `scripts/verify/m318-dashboard-command-center-depth.sh` (new)
- `scripts/verify/test-m318-dashboard-command-center-depth.sh` (new)
- `README.md`
- `specs/milestones/m318/index.md`
- `specs/3531/spec.md`
- `specs/3531/plan.md`
- `specs/3531/tasks.md`

## Risks / Mitigations
- Risk: overlap with M308/M314 causes unclear ownership across dashboard gates.
  - Mitigation: define M318 as command-center depth aggregation while M308/M314
    remain live-mutation and operator-workflow depth gates respectively.
- Risk: required-step drift can silently reduce coverage.
  - Mitigation: exact-once required-step ID checks in script/contract test.
- Risk: false-positive pass with tampered report.
  - Mitigation: verify-only mode and tamper fail-closed tests.

## Interfaces / Contracts
- Script entrypoint:
  - `scripts/verify/m318-dashboard-command-center-depth.sh`
- Report artifact:
  - `${TAU_M318_REPORT_DIR:-artifacts/dashboard-command-center-depth}/verification-report.json`
- Report keys:
  - `schema_version`, `suite_id`, `generated_at`, `overall`, `steps[]`.

## ADR
No ADR required (verification orchestration/docs scope).

## Execution Summary
1. Added RED-first contract test
   `scripts/verify/test-m318-dashboard-command-center-depth.sh` and captured
   expected pre-implementation failure while the M318 script was absent.
2. Implemented `scripts/verify/m318-dashboard-command-center-depth.sh` with
   deterministic dashboard command-center selector mapping and report emission.
3. Enforced fail-closed report validation for schema, overall consistency, and
   exact-once required-step IDs.
4. Updated README dashboard capability/gap references with M318 entrypoint.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m318-dashboard-command-center-depth.sh`
  - Result before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m318-dashboard-command-center-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m318-dashboard-command-center-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast TAU_M318_CARGO_TARGET_DIR=target-fast bash scripts/verify/m318-dashboard-command-center-depth.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/verify/m318-dashboard-command-center-depth.sh scripts/verify/test-m318-dashboard-command-center-depth.sh` passed.
  - `cargo fmt --check` passed.
