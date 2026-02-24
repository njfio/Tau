# Plan: Issue #3498 - M310 deterministic RL policy-operations depth gate

Status: Implemented

## Approach
1. Add RED-first contract test for missing M310 script.
2. Implement M310 script with deterministic RL selector mapping and JSON
   report emission.
3. Enforce fail-closed report schema + required-step checks.
4. Update README RL gap links with M310 entrypoint.
5. Run RED->GREEN->REGRESSION verification and update spec evidence.

## Affected Modules
- `scripts/verify/m310-rl-policy-ops-depth.sh` (new)
- `scripts/verify/test-m310-rl-policy-ops-depth.sh` (new)
- `README.md`
- `specs/milestones/m310/index.md`
- `specs/3498/spec.md`
- `specs/3498/plan.md`
- `specs/3498/tasks.md`

## Risks / Mitigations
- Risk: RL selector drift reduces policy-ops verification coverage.
  - Mitigation: exact-once required-step ID checks in script/contract test.
- Risk: script runtime noise from broad selectors.
  - Mitigation: use narrowly targeted deterministic selectors only.
- Risk: false-positive pass with tampered report.
  - Mitigation: verify-only mode and tamper fail-closed tests.

## Interfaces / Contracts
- Script entrypoint:
  - `scripts/verify/m310-rl-policy-ops-depth.sh`
- Report artifact:
  - `${TAU_M310_REPORT_DIR:-artifacts/rl-policy-ops-depth}/verification-report.json`
- Report keys:
  - `schema_version`, `suite_id`, `generated_at`, `overall`, `steps[]`.

## ADR
No ADR required (verification orchestration/docs scope).

## Execution Summary
1. Added RED-first contract test
   `scripts/verify/test-m310-rl-policy-ops-depth.sh` and captured expected
   pre-implementation failure while the M310 script was absent.
2. Implemented `scripts/verify/m310-rl-policy-ops-depth.sh` with
   deterministic RL selector mapping and report emission.
3. Enforced fail-closed report validation for schema, overall consistency, and
   exact-once required-step IDs.
4. Updated README true-RL capability/gap references with M310 entrypoint.
5. Fixed deterministic execution behavior by running selector commands via
   `bash -c` (non-login shell) to avoid environment-dependent login-shell
   failures in rollback checklist coverage.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m310-rl-policy-ops-depth.sh`
  - Result before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m310-rl-policy-ops-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m310-rl-policy-ops-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast bash scripts/verify/m310-rl-policy-ops-depth.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/verify/m310-rl-policy-ops-depth.sh scripts/verify/test-m310-rl-policy-ops-depth.sh` passed.
  - `cargo fmt --check` passed.
