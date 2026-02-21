# M236 - gateway endpoint constants modularization

Status: In Progress

## Context
`gateway_openresponses.rs` still owns a large endpoint/path constants block used across root and sibling modules. Moving these constants into a dedicated module reduces root-file density while preserving endpoint contracts.

## Scope
- Move endpoint/path constants from `gateway_openresponses.rs` to a dedicated constants module.
- Preserve route wiring and status payload endpoint fields.
- Ratchet root module size guard.

## Linked Issues
- Epic: #3234
- Story: #3235
- Task: #3236

## Success Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway integration_gateway_status_endpoint_returns_service_snapshot`
- `cargo test -p tau-gateway integration_gateway_status_endpoint_reports_openai_compat_runtime_counters`
- `cargo test -p tau-gateway integration_localhost_dev_mode_allows_requests_without_bearer_token`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
