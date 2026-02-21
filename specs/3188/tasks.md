# Tasks: Issue #3188 - resync stale claims in tasks/whats-missing.md

- [x] T1 (RED): create `scripts/dev/test-whats-missing.sh` with stale-marker assertions and run it expecting failure against current report.
- [x] T2 (GREEN): refresh `tasks/whats-missing.md` to current evidence-based state and update script markers.
- [x] T3 (VERIFY): run `scripts/dev/test-whats-missing.sh`, `cargo fmt --check`, `cargo clippy -- -D warnings`.
