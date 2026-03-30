# Tasks: Issue #3644 - Fix tau-gateway manual_unwrap_or clippy blocker in gateway config runtime

- [ ] T1 Red: capture the current GitHub Actions failure from PR `#3631`
      showing `tau-gateway` failing on `clippy::manual_unwrap_or`.
- [ ] T2 Green: replace the manual `match` with the equivalent
      `unwrap_or(u64::MAX)` expression. Covers C-01 and C-02.
- [ ] T3 Verify: rerun local `tau-gateway` Clippy, rerun the exact
      `fast-validate` reproduction, then push and watch PR `#3631` for
      advancement past the `tau-gateway` blocker. Covers C-03.
