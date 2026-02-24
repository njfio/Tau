# Plan: Issue #3502 - M311 deterministic TUI operator workflow depth gate

Status: Implemented

## Approach
1. Add RED-first contract test for missing M311 script.
2. Implement M311 script with deterministic TUI selector mapping and JSON
   report emission.
3. Enforce fail-closed report schema + required-step checks.
4. Update README TUI gap links with M311 entrypoint.
5. Run RED->GREEN->REGRESSION verification and update spec evidence.

## Affected Modules
- `scripts/verify/m311-tui-operator-workflow-depth.sh` (new)
- `scripts/verify/test-m311-tui-operator-workflow-depth.sh` (new)
- `README.md`
- `specs/milestones/m311/index.md`
- `specs/3502/spec.md`
- `specs/3502/plan.md`
- `specs/3502/tasks.md`

## Risks / Mitigations
- Risk: TUI selector drift reduces workflow coverage.
  - Mitigation: exact-once required-step ID checks in script/contract test.
- Risk: runtime noise from broad selectors.
  - Mitigation: use narrowly targeted deterministic test selectors.
- Risk: false-positive pass with tampered report.
  - Mitigation: verify-only mode and tamper fail-closed tests.

## Interfaces / Contracts
- Script entrypoint:
  - `scripts/verify/m311-tui-operator-workflow-depth.sh`
- Report artifact:
  - `${TAU_M311_REPORT_DIR:-artifacts/tui-operator-workflow-depth}/verification-report.json`
- Report keys:
  - `schema_version`, `suite_id`, `generated_at`, `overall`, `steps[]`.

## ADR
No ADR required (verification orchestration/docs scope).

## Execution Summary
1. Added RED-first contract test
   `scripts/verify/test-m311-tui-operator-workflow-depth.sh` and captured
   expected pre-implementation failure while the M311 script was absent.
2. Implemented `scripts/verify/m311-tui-operator-workflow-depth.sh` with
   deterministic TUI selector mapping and report emission.
3. Enforced fail-closed report validation for schema, overall consistency, and
   exact-once required-step IDs.
4. Updated README TUI capability/gap references with M311 entrypoint.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m311-tui-operator-workflow-depth.sh`
  - Result before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m311-tui-operator-workflow-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m311-tui-operator-workflow-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast bash scripts/verify/m311-tui-operator-workflow-depth.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/verify/m311-tui-operator-workflow-depth.sh scripts/verify/test-m311-tui-operator-workflow-depth.sh` passed.
  - `cargo fmt --check` passed.
