# Tasks: Issue #3639 - Contain tau-extensions deprecation lint in package-scoped clippy

- [ ] T1 Red: capture the current `tau-extensions` clippy failure under
      `-D warnings`.
- [ ] T2 Red: capture the current `fast-validate --base <sha>` failure showing
      `tau-extensions` as the next blocker on `#3631`.
- [ ] T3 Green: contain self-referential deprecated-item lint at the deprecated
      `tau-extensions` crate boundary without changing runtime behavior.
      Covers C-01 and C-02.
- [ ] T4 Verify: rerun crate-scoped clippy, one targeted test, and the exact
      `fast-validate` reproduction; then update issue/PR state. Covers C-03.
