# Spec: Issue #3228 - move gateway auth runtime-state/status types into module

Status: Implemented

## Problem Statement
`gateway_openresponses.rs` still contains auth runtime-state and auth status-report type definitions even though auth behavior is implemented in `gateway_openresponses/auth_runtime.rs`. This keeps auth-modeling concerns in the root module and slows modularization progress.

## Scope
In scope:
- Extract `GatewayAuthRuntimeState`, `GatewaySessionTokenState`, `GatewayRateLimitBucket`, and `GatewayAuthStatusReport` into `auth_runtime.rs`.
- Rewire root module to consume moved types.
- Ratchet and enforce root-module size/ownership guard.

Out of scope:
- Authentication mode semantics changes.
- API schema/path changes.
- Session/rate-limit policy changes.

## Acceptance Criteria
### AC-1 auth behavior and status endpoint behavior remain contract-stable
Given existing auth/session/rate-limit and gateway-status scenarios,
when requests are processed,
then auth issuance, expiration, rate-limit handling, and gateway status endpoint behavior remain unchanged.

### AC-2 root module ownership boundaries improve
Given refactored module layout,
when running root-module size/ownership guard,
then root line count is under tightened threshold and auth runtime-state/status type definitions are no longer in root.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | password-session auth mode fixture | `functional_gateway_auth_session_endpoint_issues_bearer_for_password_mode` | session bearer issuance behavior preserved |
| C-02 | AC-1 | Regression/Conformance | expiring session token fixture | `regression_gateway_password_session_token_expires_and_fails_closed` | expired token rejected |
| C-03 | AC-1 | Regression/Conformance | rate-limit fixture | `regression_gateway_rate_limit_rejects_excess_requests` | excess requests rejected with same behavior |
| C-04 | AC-1 | Integration/Conformance | gateway status endpoint fixture | `integration_gateway_status_endpoint_returns_service_snapshot` | status payload behavior preserved |
| C-05 | AC-2 | Functional/Regression | repo checkout | `scripts/dev/test-gateway-openresponses-size.sh` | tightened threshold + auth-type ownership guards pass |

## Success Metrics / Observable Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway functional_gateway_auth_session_endpoint_issues_bearer_for_password_mode`
- `cargo test -p tau-gateway regression_gateway_password_session_token_expires_and_fails_closed`
- `cargo test -p tau-gateway regression_gateway_rate_limit_rejects_excess_requests`
- `cargo test -p tau-gateway integration_gateway_status_endpoint_returns_service_snapshot`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
