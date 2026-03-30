# Tasks: Issue #3641 - Fix tau-onboarding clippy derivable_impls blocker

- [ ] T1 Red: capture the current `tau-onboarding` clippy failure under
      `-D warnings`.
- [ ] T2 Red: capture the current `fast-validate --base <sha>` failure showing
      `tau-onboarding` as the next blocker on `#3631`.
- [ ] T3 Green: replace the manual `Default` impl on
      `SelfImprovementConfig` with a derived default. Covers C-01 and C-02.
- [ ] T4 Verify: rerun crate-scoped clippy and the exact `fast-validate`
      reproduction; then update issue/PR state. Covers C-03.
