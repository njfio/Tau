# Tasks: Issue #3640 - Contain tau-tools extension compatibility deprecation lint

- [ ] T1 Red: capture the current `tau-tools` clippy failure under
      `-D warnings`.
- [ ] T2 Red: capture the current `fast-validate --base <sha>` failure showing
      `tau-tools` as the next blocker on `#3631`.
- [ ] T3 Green: contain deprecation lint only at the intentional
      `tau-extensions` bridge points in `tau-tools`. Covers C-01 and C-02.
- [ ] T4 Verify: rerun crate-scoped clippy, one targeted test, and the exact
      `fast-validate` reproduction; then update issue/PR state. Covers C-03.
