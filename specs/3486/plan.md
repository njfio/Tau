# Plan: Issue #3486 - M307 deterministic C5 multi-channel orchestration depth gate

Status: Implemented

## Approach
1. Add RED-first script contract test expecting missing M307 script/report.
2. Implement M307 verification script with deterministic report schema and
   required-step enforcement.
3. Map C5-01..C5-08 to stable selectors across `tau-multi-channel` and
   `tau-gateway`.
4. Update README verification links for M307 discoverability.
5. Run scoped RED->GREEN->REGRESSION verification and record evidence in tasks.

## Affected Modules
- `scripts/verify/m307-multi-channel-orchestration-depth.sh` (new)
- `scripts/verify/test-m307-multi-channel-orchestration-depth.sh` (new)
- `README.md`
- `specs/milestones/m307/index.md`
- `specs/3486/spec.md`
- `specs/3486/plan.md`
- `specs/3486/tasks.md`

## Risks / Mitigations
- Risk: selector drift silently removes C5 case coverage.
  - Mitigation: required-step IDs checked exactly-once in script contract test.
- Risk: long runtime/noise from broad selectors.
  - Mitigation: use tightly scoped deterministic test selectors only.
- Risk: shell-script regressions in CI environments.
  - Mitigation: include schema validation and mock/fail/tamper contract tests.

## Interfaces / Contracts
- Script entrypoint:
  - `scripts/verify/m307-multi-channel-orchestration-depth.sh`
- Report artifact:
  - `${TAU_M307_REPORT_DIR:-artifacts/multi-channel-orchestration-depth}/verification-report.json`
- Report schema keys:
  - `schema_version`, `suite_id`, `generated_at`, `overall`, `steps[]`.

## ADR
No ADR required (verification orchestration/docs scope only).

## Execution Summary
1. Added RED-first script contract test
   `scripts/verify/test-m307-multi-channel-orchestration-depth.sh` and captured
   expected failure while the M307 verification script was missing.
2. Implemented
   `scripts/verify/m307-multi-channel-orchestration-depth.sh` with deterministic
   C5 selector mapping and report generation.
3. Enforced fail-closed report contract checks:
   - schema/suite keys,
   - overall-step consistency,
   - exact-once required-step IDs.
4. Updated README capability-boundary and current-gap execution links to include
   M307 entrypoint.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m307-multi-channel-orchestration-depth.sh`
  - Result before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m307-multi-channel-orchestration-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m307-multi-channel-orchestration-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast bash scripts/verify/m307-multi-channel-orchestration-depth.sh` passed.
- REGRESSION evidence:
  - `bash -n scripts/verify/m307-multi-channel-orchestration-depth.sh scripts/verify/test-m307-multi-channel-orchestration-depth.sh` passed.
  - `cargo fmt --check` passed.
