# Tasks: Issue #3252 - move shell/auth entry handlers to dedicated module

- [x] T1 (RED): tighten `scripts/dev/test-gateway-openresponses-size.sh` to `650` and assert shell/auth entry handlers are not declared in root; run expecting failure.
- [x] T2 (GREEN): move `handle_webchat_page`, `handle_dashboard_shell_page`, and `handle_gateway_auth_bootstrap` into `entry_handlers.rs`; wire root imports.
- [x] T3 (VERIFY): run targeted functional tests (including existing auth bootstrap contract test) + guard.
- [x] T4 (VERIFY): run `cargo fmt --check` and `cargo clippy -- -D warnings`.
