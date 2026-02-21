# M240 - gateway shell/auth entry handlers modularization

Status: In Progress

## Context
`gateway_openresponses.rs` still includes simple shell/auth entry handlers (`handle_webchat_page`, `handle_dashboard_shell_page`, `handle_gateway_auth_bootstrap`). This route-entry glue can be isolated to continue reducing root module coupling.

## Scope
- Move shell/auth entry handlers to a dedicated module.
- Preserve route behavior and auth bootstrap contract.
- Ratchet root-module size/ownership guard.

## Linked Issues
- Epic: #3250
- Story: #3251
- Task: #3252

## Success Signals
- `scripts/dev/test-gateway-openresponses-size.sh`
- `cargo test -p tau-gateway functional_webchat_endpoint_returns_html_shell`
- `cargo test -p tau-gateway functional_dashboard_shell_endpoint_returns_html_shell`
- `cargo test -p tau-gateway functional_spec_2786_c01_gateway_auth_bootstrap_endpoint_reports_token_mode_contract`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
