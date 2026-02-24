# Plan: Issue #3490 - M308 deterministic dashboard live mutation depth gate

Status: Implemented

## Approach
1. Add RED-first script contract test that expects missing M308 script.
2. Implement M308 verification script with deterministic dashboard selector map.
3. Enforce fail-closed report schema and required-step inventory checks.
4. Update README dashboard gap references with M308 script link.
5. Run RED->GREEN->REGRESSION checks and update spec evidence.

## Affected Modules
- `scripts/verify/m308-dashboard-live-mutation-depth.sh` (new)
- `scripts/verify/test-m308-dashboard-live-mutation-depth.sh` (new)
- `README.md`
- `specs/milestones/m308/index.md`
- `specs/3490/spec.md`
- `specs/3490/plan.md`
- `specs/3490/tasks.md`

## Risks / Mitigations
- Risk: selector drift reduces coverage.
  - Mitigation: required-step IDs verified exactly-once in report.
- Risk: script runtime noise.
  - Mitigation: use targeted deterministic selectors only.
- Risk: false pass from report tampering.
  - Mitigation: script contract test includes verify-only tamper-failure path.

## Interfaces / Contracts
- Script entrypoint:
  - `scripts/verify/m308-dashboard-live-mutation-depth.sh`
- Report artifact:
  - `${TAU_M308_REPORT_DIR:-artifacts/dashboard-live-mutation-depth}/verification-report.json`
- Report keys:
  - `schema_version`, `suite_id`, `generated_at`, `overall`, `steps[]`.

## ADR
No ADR required (verification orchestration/docs scope).

## Execution Summary
1. Added RED-first contract test
   `scripts/verify/test-m308-dashboard-live-mutation-depth.sh` and captured
   expected pre-implementation failure while the M308 script was absent.
2. Implemented `scripts/verify/m308-dashboard-live-mutation-depth.sh` with
   deterministic dashboard selector mapping and report emission.
3. Added fail-closed report validation for schema, overall consistency, and
   exact-once required-step IDs.
4. Updated README dashboard gap row to include M308 verification entrypoint.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m308-dashboard-live-mutation-depth.sh`
  - Result before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m308-dashboard-live-mutation-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m308-dashboard-live-mutation-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast bash scripts/verify/m308-dashboard-live-mutation-depth.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/verify/m308-dashboard-live-mutation-depth.sh scripts/verify/test-m308-dashboard-live-mutation-depth.sh` passed.
  - `cargo fmt --check` passed.
