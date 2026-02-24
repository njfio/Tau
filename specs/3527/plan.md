# Plan: Issue #3527 - M317 deterministic TUI scenario-expansion depth gate

Status: Implemented

## Approach
1. Add RED-first contract test for missing M317 script.
2. Implement M317 script with deterministic selector mapping and JSON report
   emission.
3. Enforce fail-closed report schema + required-step checks.
4. Update README TUI gap links with M317 entrypoint.
5. Run RED->GREEN->REGRESSION verification and update spec evidence.

## Affected Modules
- `scripts/verify/m317-tui-scenario-expansion-depth.sh` (new)
- `scripts/verify/test-m317-tui-scenario-expansion-depth.sh` (new)
- `README.md`
- `specs/milestones/m317/index.md`
- `specs/3527/spec.md`
- `specs/3527/plan.md`
- `specs/3527/tasks.md`

## Risks / Mitigations
- Risk: overlap with M311 causes unclear ownership between TUI gates.
  - Mitigation: define M317 as scenario-expansion aggregation while M311 stays
    focused on operator workflow depth.
- Risk: required-step drift can silently reduce coverage.
  - Mitigation: exact-once required-step ID checks in script/contract test.
- Risk: false-positive pass with tampered report.
  - Mitigation: verify-only mode and tamper fail-closed tests.

## Interfaces / Contracts
- Script entrypoint:
  - `scripts/verify/m317-tui-scenario-expansion-depth.sh`
- Report artifact:
  - `${TAU_M317_REPORT_DIR:-artifacts/tui-scenario-expansion-depth}/verification-report.json`
- Report keys:
  - `schema_version`, `suite_id`, `generated_at`, `overall`, `steps[]`.

## ADR
No ADR required (verification orchestration/docs scope).

## Execution Summary
1. Added RED-first contract test
   `scripts/verify/test-m317-tui-scenario-expansion-depth.sh` and captured
   expected pre-implementation failure while the M317 script was absent.
2. Implemented `scripts/verify/m317-tui-scenario-expansion-depth.sh` with
   deterministic TUI scenario-expansion selector mapping and report emission.
3. Enforced fail-closed report validation for schema, overall consistency, and
   exact-once required-step IDs.
4. Updated README TUI capability/gap references with M317 entrypoint.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m317-tui-scenario-expansion-depth.sh`
  - Result before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m317-tui-scenario-expansion-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m317-tui-scenario-expansion-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast TAU_M317_CARGO_TARGET_DIR=target-fast bash scripts/verify/m317-tui-scenario-expansion-depth.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/verify/m317-tui-scenario-expansion-depth.sh scripts/verify/test-m317-tui-scenario-expansion-depth.sh` passed.
  - `cargo fmt --check` passed.
