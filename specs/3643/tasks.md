# Tasks: Issue #3643 - Fix tau-memory clippy derivable_impls blockers in memory runtime

- [ ] T1 Red: capture the current GitHub Actions failure from PR `#3631`
      showing `tau-memory` failing on `clippy::derivable_impls`.
- [ ] T2 Green: replace the manual `Default` impls on `MemoryRelationType`
      and `MemoryType` with derive-based defaults and explicit default
      variants. Covers C-01 and C-02.
- [ ] T3 Verify: rerun local `tau-memory` clippy, rerun the exact
      `fast-validate` reproduction, then push and watch PR `#3631` for
      advancement past the `tau-memory` blocker. Covers C-03.
