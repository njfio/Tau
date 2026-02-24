# M309 - Auth credential lifecycle depth verification wave

Status: Active

## Context
M309 deepens auth workflow verification by adding one deterministic gate that
aggregates provider auth matrix coverage, gateway auth-session lifecycle
contracts, integration-auth set/status/rotate/revoke flows, and secret
resolution fail-closed behavior.

Primary sources:
- `crates/tau-provider/src/integration_auth.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/auth_and_provider/provider_status_and_integration_auth.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/commands_and_packages.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Issue Hierarchy
- Epic: #3492
- Story: #3493
- Task: #3494

## Scope
- Add deterministic M309 verification script and report contract.
- Add script contract test with fail-closed required-step checks.
- Map auth credential lifecycle coverage to executable selectors.
- Update README links with M309 entrypoint.

## Exit Criteria
- `specs/3494/spec.md` is `Implemented` with AC evidence.
- M309 script report includes all required auth-lifecycle step IDs.
- Contract test fails closed on missing required-step IDs.
- README includes M309 verification entrypoint.
