# Spec: Issue #3252 - move shell/auth entry handlers to dedicated module

Status: Reviewed

## Problem Statement
`gateway_openresponses.rs` still owns simple shell/auth entry handlers, mixing route-entry glue with broader request runtime logic and limiting continued module decomposition.

## Scope
In scope:
- Move `handle_webchat_page`, `handle_dashboard_shell_page`, and `handle_gateway_auth_bootstrap` into `gateway_openresponses/entry_handlers.rs`.
- Preserve route behavior and auth bootstrap payload semantics.
- Ratchet and enforce root-module size/ownership guard.

Out of scope:
- Endpoint path changes.
- Auth/session policy behavior changes.
- Shell page markup/rendering changes.

## Acceptance Criteria
### AC-1 shell/auth entry behavior remains stable
Given existing and added functional endpoint scenarios,
when tests run,
then webchat/dashboard shell responses and auth bootstrap contract remain unchanged.

### AC-2 root module ownership boundaries improve
Given refactored module layout,
when root guard runs,
then root line count is under tightened threshold and moved shell/auth handler functions are no longer declared in root.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | gateway server fixture | `functional_webchat_endpoint_returns_html_shell` | webchat shell route behavior unchanged |
| C-02 | AC-1 | Functional/Conformance | gateway server fixture | `functional_dashboard_shell_endpoint_returns_html_shell` | dashboard shell route behavior unchanged |
| C-03 | AC-1 | Functional/Conformance | gateway server fixture | `functional_gateway_auth_bootstrap_endpoint_returns_gateway_auth_contract` | auth bootstrap payload contract remains stable |
| C-04 | AC-2 | Functional/Regression | repo checkout | `scripts/dev/test-gateway-openresponses-size.sh` | tightened threshold + ownership checks pass |

## Success Metrics / Observable Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway functional_webchat_endpoint_returns_html_shell`
- `cargo test -p tau-gateway functional_dashboard_shell_endpoint_returns_html_shell`
- `cargo test -p tau-gateway functional_gateway_auth_bootstrap_endpoint_returns_gateway_auth_contract`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
