# Plan: Issue #3470 - M303 deterministic auth-depth verification gate

Status: Implemented

## Approach
1. Add RED-first script contract expectations by defining required step IDs and
   report fields in a new test script.
2. Implement `scripts/verify/m303-auth-workflow-depth.sh` to run deterministic
   provider + gateway auth selectors and emit JSON report output.
3. Implement `scripts/verify/test-m303-auth-workflow-depth.sh` to:
   - run the M303 script,
   - assert report schema keys,
   - assert required step IDs are present,
   - fail closed when any required marker is missing.
4. Update README auth-gap execution links to include M303 script.
5. Run scoped verification commands and update spec/tasks evidence.

## Affected Modules
- `scripts/verify/m303-auth-workflow-depth.sh` (new)
- `scripts/verify/test-m303-auth-workflow-depth.sh` (new)
- `README.md`
- `specs/milestones/m303/index.md`
- `specs/3470/spec.md`
- `specs/3470/plan.md`
- `specs/3470/tasks.md`

## Risks / Mitigations
- Risk: script runtime becomes too long/noisy.
  - Mitigation: use tightly scoped selector filters and deterministic test IDs.
- Risk: selector drift silently reduces auth coverage.
  - Mitigation: enforce required step IDs in script contract test.
- Risk: report schema drift breaks downstream automation.
  - Mitigation: fixed schema keys and report assertions in test script.

## Interfaces / Contracts
- Script contract:
  - entrypoint: `scripts/verify/m303-auth-workflow-depth.sh`
  - output: `artifacts/auth-workflow-depth/verification-report.json`
  - report keys: `schema_version`, `suite_id`, `overall`, `steps[]`.
- Script test contract:
  - entrypoint: `scripts/verify/test-m303-auth-workflow-depth.sh`
  - requires all M303 step IDs in generated report.

## ADR
No ADR required (verification-only workflow/documentation scope).

## Execution Summary
1. Added RED-first contract test `scripts/verify/test-m303-auth-workflow-depth.sh`
   and captured the expected failure while `m303` script did not yet exist.
2. Implemented `scripts/verify/m303-auth-workflow-depth.sh` with deterministic
   step execution for provider auth conformance plus gateway lifecycle/edge-path
   auth selectors.
3. Implemented deterministic report emission to
   `artifacts/auth-workflow-depth/verification-report.json` with schema fields:
   `schema_version`, `suite_id`, `generated_at`, `overall`, and `steps[]`.
4. Added verify-only/fail-closed contract checks that require all M303 step IDs
   to be present exactly once in the report.
5. Updated `README.md` auth verification/gap links to include the M303 gate.

## Verification Notes
- RED evidence:
  - `bash scripts/verify/test-m303-auth-workflow-depth.sh`
  - Result before implementation:
    - `error: verification script missing or not executable: .../scripts/verify/m303-auth-workflow-depth.sh`
- GREEN evidence:
  - `bash scripts/verify/test-m303-auth-workflow-depth.sh` passed.
  - `CARGO_TARGET_DIR=target-fast bash scripts/verify/m303-auth-workflow-depth.sh` passed.
- Regression/gate evidence:
  - `bash -n scripts/verify/m303-auth-workflow-depth.sh scripts/verify/test-m303-auth-workflow-depth.sh` passed.
  - `cargo fmt --check` passed.
