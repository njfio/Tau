# Plan: Issue #3494 - M309 deterministic auth credential lifecycle depth gate

Status: Implemented

## Approach
1. Add RED-first contract test for missing M309 script.
2. Implement M309 script with deterministic auth selector mapping and JSON
   report emission.
3. Enforce fail-closed report schema + required-step checks.
4. Update README auth gap links with M309 entrypoint.
5. Run RED->GREEN->REGRESSION verification and update spec evidence.

## Affected Modules
- `scripts/verify/m309-auth-credential-lifecycle-depth.sh` (new)
- `scripts/verify/test-m309-auth-credential-lifecycle-depth.sh` (new)
- `README.md`
- `specs/milestones/m309/index.md`
- `specs/3494/spec.md`
- `specs/3494/plan.md`
- `specs/3494/tasks.md`

## Risks / Mitigations
- Risk: auth selector drift reduces coverage.
  - Mitigation: exact-once required-step ID checks in script/contract test.
- Risk: long runtime due to broad selectors.
  - Mitigation: use narrowly targeted deterministic test selectors.
- Risk: false-positive pass with tampered report.
  - Mitigation: verify-only mode and tamper fail-closed tests.

## Interfaces / Contracts
- Script entrypoint:
  - `scripts/verify/m309-auth-credential-lifecycle-depth.sh`
- Report artifact:
  - `${TAU_M309_REPORT_DIR:-artifacts/auth-credential-lifecycle-depth}/verification-report.json`
- Report keys:
  - `schema_version`, `suite_id`, `generated_at`, `overall`, `steps[]`.

## ADR
No ADR required (verification orchestration/docs scope).

## Execution Summary
1. Added RED-first script contract test
   `scripts/verify/test-m309-auth-credential-lifecycle-depth.sh` and captured
   expected pre-implementation failure while the M309 script was absent.
2. Implemented `scripts/verify/m309-auth-credential-lifecycle-depth.sh` with
   deterministic auth selector mapping and report emission.
3. Enforced fail-closed report validation for schema, overall consistency, and
   exact-once required-step IDs.
4. Updated README auth verification/gap references with M309 entrypoint.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m309-auth-credential-lifecycle-depth.sh`
  - Result before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m309-auth-credential-lifecycle-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m309-auth-credential-lifecycle-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast bash scripts/verify/m309-auth-credential-lifecycle-depth.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/verify/m309-auth-credential-lifecycle-depth.sh scripts/verify/test-m309-auth-credential-lifecycle-depth.sh` passed.
  - `cargo fmt --check` passed.
