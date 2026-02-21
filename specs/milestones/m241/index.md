# M241 - gateway request preflight helpers modularization

Status: In Progress

## Context
`gateway_openresponses.rs` still contains request preflight helper utilities for authorization, payload validation/parsing, and policy gate construction. These shared helpers can be isolated to reduce root-module coupling.

## Scope
- Move request preflight helper functions into a dedicated module.
- Preserve openresponses preflight/runtime behavior.
- Ratchet root-module size/ownership guard.

## Linked Issues
- Epic: #3254
- Story: #3255
- Task: #3256

## Success Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway integration_spec_c01_openresponses_preflight_blocks_over_budget_request`
- `cargo test -p tau-gateway integration_spec_c02_openresponses_preflight_skips_provider_dispatch`
- `cargo test -p tau-gateway regression_spec_c03_openresponses_preflight_preserves_success_schema`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
