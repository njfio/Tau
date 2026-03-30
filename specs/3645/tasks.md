# Tasks: Issue #3645 - Fix tau-onboarding cmp_owned clippy blocker in startup transport modes

- [x] T1 Red: capture the current GitHub Actions failure from PR `#3631`
      showing `tau-onboarding` failing on `clippy::cmp_owned`.
- [x] T2 Green: replace the owned `PathBuf::from(...)` comparison with the
      equivalent borrowed-path comparison. Covers C-01 and C-02.
- [x] T3 Verify: rerun local `tau-onboarding` Clippy, rerun the exact
      `fast-validate` reproduction, then push and watch PR `#3631` for
      advancement past the `tau-onboarding` blocker. Covers C-03.
