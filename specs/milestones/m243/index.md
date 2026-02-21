# M243 - gateway stream response handler modularization

Status: In Progress

## Context
`gateway_openresponses.rs` still owns the `stream_openresponses` SSE handler. Extracting this helper keeps root focused on route orchestration and request execution while tightening ownership boundaries.

## Scope
- Move `stream_openresponses` into a dedicated module.
- Preserve openresponses stream/non-stream contracts.
- Ratchet root-module size/ownership guard.

## Linked Issues
- Epic: #3262
- Story: #3263
- Task: #3264

## Success Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway functional_openresponses_endpoint_returns_non_stream_response`
- `cargo test -p tau-gateway functional_openresponses_endpoint_streams_sse_for_stream_true`
- `cargo test -p tau-gateway regression_openresponses_endpoint_rejects_malformed_json_body`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
