# Tasks: Issue #3268 - move auth-session handler to dedicated module

- [ ] T1 (RED): tighten `scripts/dev/test-gateway-openresponses-size.sh` threshold and assert `handle_gateway_auth_session` is not declared in root; run expecting failure.
- [ ] T2 (GREEN): move `handle_gateway_auth_session` from `gateway_openresponses.rs` into `auth_session_handler.rs`; wire root imports.
- [ ] T3 (VERIFY): run targeted auth-session conformance tests + guard.
- [ ] T4 (VERIFY): run `cargo fmt --check` and `cargo clippy -- -D warnings`.
