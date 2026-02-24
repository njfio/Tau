# Plan: Issue #3518 - M315 deterministic E2E operator-route scenario depth gate

Status: Implemented

## Approach
1. Add RED-first contract test for missing M315 script.
2. Implement M315 script with deterministic operator-route selector mapping and
   JSON report emission.
3. Enforce fail-closed report schema + required-step checks.
4. Update README E2E gap links with M315 entrypoint.
5. Run RED->GREEN->REGRESSION verification and update spec evidence.

## Affected Modules
- `scripts/verify/m315-e2e-operator-route-depth.sh` (new)
- `scripts/verify/test-m315-e2e-operator-route-depth.sh` (new)
- `README.md`
- `specs/milestones/m315/index.md`
- `specs/3518/spec.md`
- `specs/3518/plan.md`
- `specs/3518/tasks.md`

## Risks / Mitigations
- Risk: selector drift reduces operator-route depth coverage.
  - Mitigation: exact-once required-step ID checks in script/contract test.
- Risk: runtime noise from broad selectors.
  - Mitigation: use narrowly scoped deterministic selector names.
- Risk: false-positive pass with tampered report.
  - Mitigation: verify-only mode and tamper fail-closed tests.

## Interfaces / Contracts
- Script entrypoint:
  - `scripts/verify/m315-e2e-operator-route-depth.sh`
- Report artifact:
  - `${TAU_M315_REPORT_DIR:-artifacts/e2e-operator-route-depth}/verification-report.json`
- Report keys:
  - `schema_version`, `suite_id`, `generated_at`, `overall`, `steps[]`.

## ADR
No ADR required (verification orchestration/docs scope).

## Execution Summary
1. Added RED-first contract test
   `scripts/verify/test-m315-e2e-operator-route-depth.sh` and captured expected
   pre-implementation failure while the M315 script was absent.
2. Implemented `scripts/verify/m315-e2e-operator-route-depth.sh` with
   deterministic operator-route selector mapping and report emission.
3. Enforced fail-closed report validation for schema, overall consistency, and
   exact-once required-step IDs.
4. Updated README E2E capability/gap references with M315 entrypoint.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m315-e2e-operator-route-depth.sh`
  - Result before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m315-e2e-operator-route-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m315-e2e-operator-route-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast bash scripts/verify/m315-e2e-operator-route-depth.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/verify/m315-e2e-operator-route-depth.sh scripts/verify/test-m315-e2e-operator-route-depth.sh` passed.
  - `cargo fmt --check` passed.
