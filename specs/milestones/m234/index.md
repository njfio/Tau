# M234 - gateway auth state type modularization

Status: In Progress

## Context
`gateway_openresponses.rs` still owns auth runtime-state and auth status-report type definitions that are only used by `auth_runtime.rs` and server-state wiring. Moving these types into `auth_runtime.rs` keeps auth concerns localized and continues root-module decomposition.

## Scope
- Move auth runtime-state/status types into `gateway_openresponses/auth_runtime.rs`.
- Preserve auth/session/rate-limit/status behavior.
- Ratchet root-module size guard.

## Linked Issues
- Epic: #3226
- Story: #3227
- Task: #3228

## Success Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway functional_gateway_auth_session_endpoint_issues_bearer_for_password_mode`
- `cargo test -p tau-gateway regression_gateway_password_session_token_expires_and_fails_closed`
- `cargo test -p tau-gateway regression_gateway_rate_limit_rejects_excess_requests`
- `cargo test -p tau-gateway integration_gateway_status_endpoint_returns_service_snapshot`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
