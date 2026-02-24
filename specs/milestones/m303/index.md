# M303 - Auth workflow depth verification wave

Status: Active

## Context
M303 expands deterministic auth verification depth so operator gates cover the
provider auth matrix plus gateway lifecycle and edge-path contracts across token
mode, password-session mode, and localhost-dev mode.

Primary sources:
- `scripts/verify/m296-live-auth-validation.sh`
- `scripts/verify/m295-operator-maturity-wave.sh`
- `crates/tau-provider/tests/auth_workflow_conformance.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Issue Hierarchy
- Epic: #3468
- Story: #3469
- Task: #3470

## Scope
- Add a dedicated deterministic auth-depth verification script for operators.
- Include gateway auth lifecycle and edge-path selectors beyond baseline
  session-issuance checks.
- Add a verification script contract test to keep selector inventory stable.
- Update README auth-gap execution links with the new verification gate.

## Exit Criteria
- `specs/3470/spec.md` is implemented with AC-to-test evidence.
- New auth-depth verification script passes and emits deterministic report data.
- Script test verifies required selector coverage and fail-closed behavior.
- README links include M303 auth-depth verification entrypoint.
