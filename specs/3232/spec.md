# Spec: Issue #3232 - move gateway tool registrar api types into module

Status: Reviewed

## Problem Statement
`gateway_openresponses.rs` still owns tool registrar public API definitions that are orthogonal to request routing/runtime logic. This keeps root module size high and mixes API type definitions with server internals.

## Scope
In scope:
- Move `GatewayToolRegistrar`, `NoopGatewayToolRegistrar`, and `GatewayToolRegistrarFn` into `tool_registrar.rs`.
- Re-export these items from root module to preserve API path stability.
- Ratchet and enforce root-module size/ownership guard.

Out of scope:
- Behavior/signature changes for tool registration.
- Endpoint behavior changes.

## Acceptance Criteria
### AC-1 root API stability is preserved
Given existing gateway runtime/test call sites,
when building and running focused integration scenarios,
then tool registrar behavior and root API usage remain unchanged.

### AC-2 root module ownership boundaries improve
Given refactored module layout,
when running root-module size/ownership guard,
then root line count is under tightened threshold and tool registrar API definitions are no longer declared in root.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Integration/Conformance | default gateway server fixture | `integration_gateway_status_endpoint_returns_service_snapshot` | gateway still composes/configures with root API symbols intact |
| C-02 | AC-1 | Integration/Conformance | localhost-dev auth fixture | `integration_localhost_dev_mode_allows_requests_without_bearer_token` | openresponses flow with tool registrar config remains stable |
| C-03 | AC-2 | Functional/Regression | repo checkout | `scripts/dev/test-gateway-openresponses-size.sh` | tightened threshold + tool-registrar ownership guards pass |

## Success Metrics / Observable Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway integration_gateway_status_endpoint_returns_service_snapshot`
- `cargo test -p tau-gateway integration_localhost_dev_mode_allows_requests_without_bearer_token`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
