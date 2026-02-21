# Tasks: Issue #3184 - deterministic directory-path rejection for browser DID report writer

- [ ] T1 (RED): add C-01 test in `crates/kamn-sdk/src/lib.rs` and run `cargo test -p kamn-sdk spec_3184 -- --test-threads=1` expecting failure.
- [ ] T2 (GREEN): implement minimal directory-path guard and rerun targeted conformance tests.
- [ ] T3 (VERIFY): run `cargo test -p kamn-sdk`, `cargo fmt --check`, `cargo clippy -p kamn-sdk -- -D warnings`.
