# Spec: Issue #3260 - move websocket and stream handlers to dedicated module

Status: Reviewed

## Problem Statement
`gateway_openresponses.rs` still owns WebSocket/session stream handlers (`handle_gateway_ws_upgrade`, `run_dashboard_stream_loop`) that can be isolated without behavior change. Keeping them in root increases module density and weakens ownership boundaries.

## Scope
In scope:
- Move ws/stream helper handlers from root into `gateway_openresponses/ws_stream_handlers.rs`.
- Preserve ws auth/session semantics and dashboard stream output contracts.
- Ratchet and enforce root-module size/ownership guard.

Out of scope:
- Endpoint path or payload contract changes.
- Authentication model changes.
- Event-stream schema changes.

## Acceptance Criteria
### AC-1 websocket and stream contracts remain stable
Given existing ws/stream functional tests,
when tests run,
then ws upgrade auth/token behavior and dashboard stream behavior remain unchanged.

### AC-2 root module ownership boundaries improve
Given refactored module layout,
when root guard runs,
then root line count is under tightened threshold and moved ws/stream function definitions are no longer declared in root.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | missing auth token ws upgrade request | `functional_gateway_ws_endpoint_rejects_unauthorized_upgrade` | ws upgrade is rejected with unauthorized response |
| C-02 | AC-1 | Functional/Conformance | authorized ws capability/ping request | `functional_gateway_ws_endpoint_supports_capabilities_and_ping_pong` | ws capability and ping/pong behavior remains stable |
| C-03 | AC-1 | Functional/Conformance | dashboard stream reconnect request | `integration_dashboard_stream_supports_reconnect_reset_and_snapshot_updates` | stream endpoint emits reset and snapshot updates for reconnect flow |
| C-04 | AC-1 | Regression/Conformance | ws session status + reset roundtrip request | `integration_gateway_ws_session_status_and_reset_roundtrip` | ws session status/reset behavior remains stable |
| C-05 | AC-2 | Functional/Regression | repo checkout | `scripts/dev/test-gateway-openresponses-size.sh` | tightened threshold + ownership checks pass |

## Success Metrics / Observable Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway functional_gateway_ws_endpoint_rejects_unauthorized_upgrade`
- `cargo test -p tau-gateway functional_gateway_ws_endpoint_supports_capabilities_and_ping_pong`
- `cargo test -p tau-gateway integration_dashboard_stream_supports_reconnect_reset_and_snapshot_updates`
- `cargo test -p tau-gateway integration_gateway_ws_session_status_and_reset_roundtrip`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
