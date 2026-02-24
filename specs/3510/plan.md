# Plan: Issue #3510 - M313 deterministic E2E core scenario depth gate

Status: Implemented

## Approach
1. Add RED-first contract test for missing M313 script.
2. Implement M313 script with deterministic integration+gateway selector
   mapping and JSON report emission.
3. Enforce fail-closed report schema + required-step checks.
4. Update README integration/gap links with M313 entrypoint.
5. Run RED->GREEN->REGRESSION verification and update spec evidence.

## Affected Modules
- `scripts/verify/m313-e2e-core-scenario-depth.sh` (new)
- `scripts/verify/test-m313-e2e-core-scenario-depth.sh` (new)
- `README.md`
- `specs/milestones/m313/index.md`
- `specs/3510/spec.md`
- `specs/3510/plan.md`
- `specs/3510/tasks.md`

## Risks / Mitigations
- Risk: selector drift reduces E2E core coverage.
  - Mitigation: exact-once required-step ID checks in script/contract test.
- Risk: runtime noise from broad selectors.
  - Mitigation: use scoped deterministic selector names only.
- Risk: false-positive pass with tampered report.
  - Mitigation: verify-only mode and tamper fail-closed tests.

## Interfaces / Contracts
- Script entrypoint:
  - `scripts/verify/m313-e2e-core-scenario-depth.sh`
- Report artifact:
  - `${TAU_M313_REPORT_DIR:-artifacts/e2e-core-scenario-depth}/verification-report.json`
- Report keys:
  - `schema_version`, `suite_id`, `generated_at`, `overall`, `steps[]`.

## ADR
No ADR required (verification orchestration/docs scope).

## Execution Summary
1. Added RED-first contract test
   `scripts/verify/test-m313-e2e-core-scenario-depth.sh` and captured expected
   pre-implementation failure while the M313 script was absent.
2. Implemented `scripts/verify/m313-e2e-core-scenario-depth.sh` with
   deterministic integration+gateway selector mapping and report emission.
3. Enforced fail-closed report validation for schema, overall consistency, and
   exact-once required-step IDs.
4. Updated README capability/gap references with M313 entrypoint.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m313-e2e-core-scenario-depth.sh`
  - Result before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m313-e2e-core-scenario-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m313-e2e-core-scenario-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast bash scripts/verify/m313-e2e-core-scenario-depth.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/verify/m313-e2e-core-scenario-depth.sh scripts/verify/test-m313-e2e-core-scenario-depth.sh` passed.
  - `cargo fmt --check` passed.
