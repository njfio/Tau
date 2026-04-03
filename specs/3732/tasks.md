# Tasks: Issue #3732 - Immutable safety floor for gateway safety endpoints

- [x] T1 (RED): add floor tests for disabling secret-leak detection and outbound
      payload scanning.
- [x] T2 (RED): add floor tests for modified default rule definitions.
- [x] T3 (GREEN): strengthen policy floor invariants in `tau-safety`.
- [x] T4 (GREEN): strengthen default-rule floor enforcement in `tau-safety`.
- [x] T5 (VERIFY): run targeted `tau-safety` regressions plus `cargo check -p
      tau-gateway` and `cargo fmt --check`.
