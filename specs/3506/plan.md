# Plan: Issue #3506 - M312 deterministic auth live-env validation depth gate

Status: Implemented

## Approach
1. Add RED-first contract test for `m296-live-auth-validation.sh`.
2. Add RED-first contract test for missing M312 script.
3. Implement mock-mode support in `m296-live-auth-validation.sh`.
4. Implement M312 script with deterministic auth selector mapping and JSON
   report emission.
5. Enforce fail-closed report schema + required-step checks.
6. Update README auth gap links with M312 entrypoint.
7. Run RED->GREEN->REGRESSION verification and update spec evidence.

## Affected Modules
- `scripts/verify/m296-live-auth-validation.sh`
- `scripts/verify/test-m296-live-auth-validation.sh` (new)
- `scripts/verify/m312-auth-live-env-depth.sh` (new)
- `scripts/verify/test-m312-auth-live-env-depth.sh` (new)
- `README.md`
- `specs/milestones/m312/index.md`
- `specs/3506/spec.md`
- `specs/3506/plan.md`
- `specs/3506/tasks.md`

## Risks / Mitigations
- Risk: live-env script changes alter existing skip semantics.
  - Mitigation: explicit skip-contract coverage in new test script.
- Risk: selector drift reduces auth depth coverage.
  - Mitigation: exact-once required-step ID checks in script/contract test.
- Risk: false-positive pass with tampered report.
  - Mitigation: verify-only mode and tamper fail-closed tests.

## Interfaces / Contracts
- Live-auth script env controls:
  - `TAU_M296_AUTH_LIVE_ENABLE`
  - `TAU_PROVIDER_KEYS_FILE`
  - `TAU_M296_AUTH_LIVE_MOCK_MODE`
  - `TAU_M296_AUTH_LIVE_MOCK_FAIL`
- M312 script entrypoint:
  - `scripts/verify/m312-auth-live-env-depth.sh`
- M312 report artifact:
  - `${TAU_M312_REPORT_DIR:-artifacts/auth-live-env-depth}/verification-report.json`
- Report keys:
  - `schema_version`, `suite_id`, `generated_at`, `overall`, `steps[]`.

## ADR
No ADR required (verification orchestration/docs scope).

## Execution Summary
1. Added RED-first contract test
   `scripts/verify/test-m296-live-auth-validation.sh` and captured expected
   failure before mock-mode support existed.
2. Added RED-first contract test
   `scripts/verify/test-m312-auth-live-env-depth.sh` and captured expected
   missing-script failure before M312 implementation.
3. Implemented mock-mode support in
   `scripts/verify/m296-live-auth-validation.sh` for deterministic pass/fail
   coverage without live network dependency.
4. Implemented `scripts/verify/m312-auth-live-env-depth.sh` with deterministic
   selector mapping and report emission.
5. Enforced fail-closed report validation for schema, overall consistency, and
   exact-once required-step IDs.
6. Updated README auth verification/gap references with M312 entrypoint.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m296-live-auth-validation.sh`
  - Expected failure before implementation:
    - `assertion failed (mock mode pass exit code): expected '0' got '20'`
  - `bash scripts/verify/test-m312-auth-live-env-depth.sh`
  - Expected failure before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m312-auth-live-env-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m296-live-auth-validation.sh` passed.
  - `bash scripts/verify/test-m312-auth-live-env-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast bash scripts/verify/m312-auth-live-env-depth.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/verify/m296-live-auth-validation.sh scripts/verify/test-m296-live-auth-validation.sh scripts/verify/m312-auth-live-env-depth.sh scripts/verify/test-m312-auth-live-env-depth.sh` passed.
  - `cargo fmt --check` passed.
