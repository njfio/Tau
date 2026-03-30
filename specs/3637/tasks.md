# Tasks: Issue #3637 - Resolve tau-coding-agent warning debt exposed by package-scoped clippy

- [ ] T1 Red: capture the current clippy failures for the impacted branch with
      `cargo clippy -p tau-coding-agent --all-targets --all-features --no-deps -- -D warnings`.
- [ ] T2 Red: capture the current `fast-validate --base <sha>` failure after
      the `#3636` fmt-scope fix.
- [ ] T3 Green: replace the manual `Default` impl in `tau-safety` with the
      derive-based form expected by clippy. Covers C-01.
- [ ] T4 Green: contain intentional deprecated `tau_extensions` compatibility
      usage in `tau-coding-agent` with narrow, explicit allowances. Covers C-02.
- [ ] T5 Green: contain intentional staged self-modification dead-code with
      narrow allowances or equivalent wiring-safe fixes. Covers C-03.
- [ ] T6 Verify: rerun scoped clippy and the exact `fast-validate` reproduction,
      then update issue/PR state. Covers C-04.
