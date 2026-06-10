# Tasks 3811: Actionable Verifier Missing Inputs

- [x] T1 (RED): Add tests for missing filter and missing package verifier-plan
  diagnostics.
- [x] T2 (GREEN): Return repo-aware verifier derivation diagnostics.
- [x] T3 (GREEN): Merge diagnostics into `AutonomousCodingVerifierPlan`.
- [x] T4 (VERIFY): Run focused tests, runtime module tests, clippy, fmt, and
  diff hygiene.

## Evidence

- RED: `CARGO_TARGET_DIR=/tmp/rust_pi-3811-target cargo test -p tau-runtime spec_3811 -- --test-threads=1` failed because the verifier plan did not name the exact missing filter/package inputs.
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3811-target cargo test -p tau-runtime spec_3811 -- --test-threads=1` passed: 2 passed.
- REGRESSION: `CARGO_TARGET_DIR=/tmp/rust_pi-3811-target cargo test -p tau-runtime spec_3807_c03 -- --test-threads=1` passed after gating diagnostics to missing-verifier authority.
- VERIFY: `cargo fmt --check` passed.
- VERIFY: `python3 .github/scripts/oversized_file_guard.py --repo-root . --default-threshold 4000 --exemptions-file tasks/policies/oversized-file-exemptions.json --policy-guide docs/guides/oversized-file-policy.md --json-output-file /tmp/rust_pi-3811-oversized-file-guard.json` passed.
- VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3811-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` passed: 28 passed.
- VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3811-target cargo clippy -p tau-runtime --lib --tests -- -D warnings` passed.
- VERIFY: `git diff --check` passed.
