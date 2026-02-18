# Tasks #2487

1. T1 (RED, conformance): add failing C-01..C-04 tests in `crates/tau-runtime/src/heartbeat_runtime.rs`.
2. T2 (GREEN, implementation): add `ArcSwap` active config + `notify` policy watcher and TOML policy parsing.
3. T3 (REFACTOR): remove/replace old JSON fingerprint polling helpers while preserving diagnostics.
4. T4 (REGRESSION): run scoped heartbeat runtime regression tests.
5. T5 (VERIFY): run fmt/clippy/scoped tests + mutation test for in-diff changes.

## Test Tier Mapping
- Unit: helper validation/parsing and path resolution.
- Functional: runtime scheduler heartbeat cycle with active config swaps.
- Conformance: C-01..C-04 spec tests.
- Integration: end-to-end running scheduler policy update behavior.
- Regression: no-change and invalid-policy fail-closed behavior.
