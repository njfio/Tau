# M242 - gateway websocket-stream handlers modularization

Status: In Progress

## Context
`gateway_openresponses.rs` still contains WebSocket upgrade and dashboard stream loop handlers. Extracting these handlers into a dedicated module keeps root focused on routing and request orchestration.

## Scope
- Move WebSocket upgrade and dashboard stream-loop helpers into a dedicated module.
- Preserve ws/dashboard stream behavior and contracts.
- Ratchet root-module size/ownership guard.

## Linked Issues
- Epic: #3258
- Story: #3259
- Task: #3260

## Success Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway functional_gateway_ws_endpoint_rejects_unauthorized_upgrade`
- `cargo test -p tau-gateway functional_gateway_ws_endpoint_supports_capabilities_and_ping_pong`
- `cargo test -p tau-gateway integration_dashboard_stream_supports_reconnect_reset_and_snapshot_updates`
- `cargo test -p tau-gateway integration_gateway_ws_session_status_and_reset_roundtrip`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
