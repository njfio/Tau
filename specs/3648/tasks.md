# Tasks: Issue #3648 - Fix tau-browser-automation live fixture launcher race in CI

- [x] T1 Red: capture the current GitHub Actions failure from PR `#3631`
      showing `integration_live_fixture_maps_executor_failures_to_retryable_backend_unavailable`
      failing with `Text file busy (os error 26)`.
- [x] T2 Green: replace the unstable shell-script temporary executor fixture
      with a CI-stable executable helper while preserving the same failure
      mapping assertions. Covers C-01 and C-02.
- [x] T3 Verify: rerun the focused tau-browser-automation selector, rerun the
      relevant crate tests, rerun the exact `fast-validate` reproduction, then
      push and watch PR `#3631` for advancement past the launcher race. Covers
      C-03.
