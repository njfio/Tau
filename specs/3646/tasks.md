# Tasks: Issue #3646 - Fix tau-slack-runtime filter_next clippy blocker in send-file URL handling

- [x] T1 Red: capture the current GitHub Actions failure from PR `#3631`
      showing `tau-slack-runtime` failing on `clippy::filter_next`.
- [x] T2 Green: replace the `filter(...).next_back()` pattern with the
      equivalent `rfind(...)` expression. Covers C-01 and C-02.
- [x] T3 Verify: rerun local `tau-slack-runtime` Clippy, rerun the exact
      `fast-validate` reproduction, then push and watch PR `#3631` for
      advancement past the `tau-slack-runtime` blocker. Covers C-03.
