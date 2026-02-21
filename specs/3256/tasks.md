# Tasks: Issue #3256 - move openresponses preflight helpers to dedicated module

- [x] T1 (RED): tighten `scripts/dev/test-gateway-openresponses-size.sh` to `600` and assert moved preflight helpers are not declared in root; run expecting failure.
- [x] T2 (GREEN): move preflight helper functions from `gateway_openresponses.rs` into `request_preflight.rs`; wire root imports.
- [x] T3 (VERIFY): run targeted preflight conformance tests + guard.
- [x] T4 (VERIFY): run `cargo fmt --check` and `cargo clippy -- -D warnings`.
