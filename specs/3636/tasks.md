# Tasks: Issue #3636 - Scope fast-validate fmt checks to changed Rust surface

- [x] T1 Red: extend `scripts/dev/test-fast-validate.sh` with a regression case
      where unrelated workspace fmt drift currently fails a narrow PR-scoped
      run. Covers C-03.
- [x] T2 Red: add a regression case proving an unformatted changed Rust file in
      scope still fails. Covers C-04.
- [x] T3 Mark the P1 spec bundle reviewed and wire `#3636` into M330. Covers
      AC-1 through AC-4.
- [x] T4 Green: update `scripts/dev/fast-validate.sh` so PR-scoped fmt uses
      changed Rust files while full-workspace modes keep `cargo fmt --all`.
      Covers C-01 / C-02.
- [x] T5 Green: rerun `./scripts/dev/test-fast-validate.sh` and the exact
      `#3631` reproduction command to confirm the blocker is gone.
- [ ] T6 Push the updated `#3631` branch and recheck CI. Covers delivery.
