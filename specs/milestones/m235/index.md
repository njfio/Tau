# M235 - gateway tool registrar api modularization

Status: In Progress

## Context
`gateway_openresponses.rs` still contains public tool-registrar API definitions (`GatewayToolRegistrar`, `NoopGatewayToolRegistrar`, `GatewayToolRegistrarFn`). These can be isolated into a dedicated module while preserving root-level API surface via re-exports.

## Scope
- Move tool registrar trait/public structs to `gateway_openresponses/tool_registrar.rs`.
- Re-export those items from `gateway_openresponses.rs` to preserve API stability.
- Ratchet root-module size guard.

## Linked Issues
- Epic: #3230
- Story: #3231
- Task: #3232

## Success Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway integration_gateway_status_endpoint_returns_service_snapshot`
- `cargo test -p tau-gateway integration_localhost_dev_mode_allows_requests_without_bearer_token`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
