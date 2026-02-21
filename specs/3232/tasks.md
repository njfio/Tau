# Tasks: Issue #3232 - move gateway tool registrar api types into module

- [x] T1 (RED): tighten `scripts/dev/test-gateway-openresponses-size.sh` to `1195` and add guard that tool registrar API definitions are not in root; run expecting failure.
- [x] T2 (GREEN): move tool registrar trait/public structs/impls from `gateway_openresponses.rs` to `tool_registrar.rs` and re-export from root.
- [x] T3 (VERIFY): run size guard, focused integration tests, `cargo fmt --check`, and `cargo clippy -- -D warnings`.
