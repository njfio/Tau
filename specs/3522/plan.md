# Plan: Issue #3522 - M316 deterministic true RL operations drill-depth gate

Status: Implemented

## Approach
1. Add RED-first contract test for missing M316 script.
2. Implement M316 script with deterministic selector mapping and JSON report
   emission.
3. Enforce fail-closed report schema + required-step checks.
4. Update README true RL gap links with M316 entrypoint.
5. Run RED->GREEN->REGRESSION verification and update spec evidence.

## Affected Modules
- `scripts/verify/m316-rl-operations-drill-depth.sh` (new)
- `scripts/verify/test-m316-rl-operations-drill-depth.sh` (new)
- `README.md`
- `specs/milestones/m316/index.md`
- `specs/3522/spec.md`
- `specs/3522/plan.md`
- `specs/3522/tasks.md`

## Risks / Mitigations
- Risk: overlap with M310 causes unclear ownership between RL depth gates.
  - Mitigation: define M316 as drill-depth aggregation (M24 operational drills)
    and keep M310 as policy-ops depth gate.
- Risk: required-step drift can silently reduce coverage.
  - Mitigation: exact-once required-step ID checks in script/contract test.
- Risk: false-positive pass with tampered report.
  - Mitigation: verify-only mode and tamper fail-closed tests.

## Interfaces / Contracts
- Script entrypoint:
  - `scripts/verify/m316-rl-operations-drill-depth.sh`
- Report artifact:
  - `${TAU_M316_REPORT_DIR:-artifacts/rl-operations-drill-depth}/verification-report.json`
- Report keys:
  - `schema_version`, `suite_id`, `generated_at`, `overall`, `steps[]`.

## ADR
No ADR required (verification orchestration/docs scope).

## Execution Summary
1. Added RED-first contract test
   `scripts/verify/test-m316-rl-operations-drill-depth.sh` and captured expected
   pre-implementation failure while the M316 script was absent.
2. Implemented `scripts/verify/m316-rl-operations-drill-depth.sh` with
   deterministic true-RL drill selector mapping and report emission.
3. Enforced fail-closed report validation for schema, overall consistency, and
   exact-once required-step IDs.
4. Updated README true RL capability/gap references with M316 entrypoint.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m316-rl-operations-drill-depth.sh`
  - Result before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m316-rl-operations-drill-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m316-rl-operations-drill-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast TAU_M316_CARGO_TARGET_DIR=target-fast bash scripts/verify/m316-rl-operations-drill-depth.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/verify/m316-rl-operations-drill-depth.sh scripts/verify/test-m316-rl-operations-drill-depth.sh` passed.
  - `cargo fmt --check` passed.
