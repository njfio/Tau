# Tasks: Issue #3168 - kamn-core label boundary validation hardening

- [ ] T1 (RED): add C-01..C-04 tests in `crates/kamn-core/src/lib.rs` and run `cargo test -p kamn-core spec_3168 -- --test-threads=1` expecting at least one failure.
- [ ] T2 (GREEN): implement boundary validation guard in `normalize_identifier` and re-run targeted tests to pass.
- [ ] T3 (VERIFY): run `cargo test -p kamn-core`, `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`.
