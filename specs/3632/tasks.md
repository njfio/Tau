# Tasks: Issue #3632 - Make fast-validate resilient to shallow PR history

- [x] T1 Red: extend `scripts/dev/test-fast-validate.sh` with a deterministic
      shallow-history regression case that fails on current behavior. Covers
      C-01 / C-04.
- [x] T2 Create milestone/spec artifacts for `#3632` and mark the P1 scope
      reviewed before implementation. Covers AC-1 through AC-4.
- [x] T3 Update `scripts/dev/fast-validate.sh` to use a bounded two-dot diff
      fallback when the merge base is unavailable locally. Covers C-02 / AC-2.
- [x] T4 Emit explicit warning output for the fallback path instead of leaking
      raw git fatal noise. Covers C-03 / AC-3.
- [x] T5 Green: rerun `./scripts/dev/test-fast-validate.sh` and confirm the
      regression case passes. Covers C-01 through C-04.
- [x] T6 Update issue/PR state, commit, push, and open a dedicated PR. Covers
      delivery/closure.
