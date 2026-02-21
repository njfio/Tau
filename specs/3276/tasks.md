# Tasks: Issue #3276 - move openresponses execution handler to dedicated module

- [ ] T1 (RED): tighten `scripts/dev/test-gateway-openresponses-size.sh` threshold and assert `execute_openresponses_request` is not declared in root; run expecting failure.
- [ ] T2 (GREEN): move `execute_openresponses_request` from `gateway_openresponses.rs` into `openresponses_execution_handler.rs`; wire root imports.
- [ ] T3 (VERIFY): run targeted openresponses execution conformance tests + guard.
- [ ] T4 (VERIFY): run `cargo fmt --check` and `cargo clippy -- -D warnings`.
