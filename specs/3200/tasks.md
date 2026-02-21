# Tasks: Issue #3200 - reconcile stale dashboard gap claim in whats-missing report

- [x] T1 (RED): update `scripts/dev/test-whats-missing.sh` dashboard marker assertions and run expecting failure.
- [x] T2 (GREEN): update `tasks/whats-missing.md` dashboard status wording to resolved state and rerun conformance scripts.
- [x] T3 (VERIFY): run `scripts/dev/verify-dashboard-consolidation.sh`, `scripts/dev/test-whats-missing.sh`, `cargo fmt --check`, `cargo clippy -- -D warnings`.
