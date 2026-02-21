# Tasks: Issue #3264 - move stream_openresponses handler to dedicated module

- [ ] T1 (RED): tighten `scripts/dev/test-gateway-openresponses-size.sh` threshold and assert `stream_openresponses` is not declared in root; run expecting failure.
- [ ] T2 (GREEN): move `stream_openresponses` from `gateway_openresponses.rs` into `stream_response_handler.rs`; wire root imports.
- [ ] T3 (VERIFY): run targeted openresponses conformance tests + guard.
- [ ] T4 (VERIFY): run `cargo fmt --check` and `cargo clippy -- -D warnings`.
