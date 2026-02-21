# Tasks: Issue #3228 - move gateway auth runtime-state/status types into module

- [x] T1 (RED): tighten `scripts/dev/test-gateway-openresponses-size.sh` to `1230` and add guard that auth types are not in root; run expecting failure.
- [x] T2 (GREEN): move auth runtime-state/status types from `gateway_openresponses.rs` to `auth_runtime.rs`; rewire visibility/imports.
- [x] T3 (VERIFY): run size guard, targeted auth/status tests, `cargo fmt --check`, and `cargo clippy -- -D warnings`.
