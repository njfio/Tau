# Tasks 3809: Derived Docs Verifier for Issue-to-Merge

- [x] T1 (RED): Add failing runtime tests for concrete docs verifier derivation
  and missing-marker fail-closed behavior.
- [x] T2 (GREEN): Derive safe docs verifier commands from quoted/backticked
  single-token markers.
- [x] T3 (GREEN): Persist the intake record for auto-derived verifier runs.
- [x] T4 (DOCS): Document the narrow derived-docs verifier boundary.
- [x] T5 (VERIFY): Run focused runtime tests, module tests, clippy, fmt, and
  diff hygiene.

## Evidence

- RED: `CARGO_TARGET_DIR=/tmp/rust_pi-3809-target cargo test -p tau-runtime spec_3809 -- --test-threads=1` failed before implementation with `Blocked` instead of `PrReady`.
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3809-target cargo test -p tau-runtime spec_3809 -- --test-threads=1` passed.
- VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3809-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` passed.
- VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3809-target cargo clippy -p tau-runtime --lib --tests -- -D warnings` passed.
- VERIFY: `cargo fmt --check` passed.
- VERIFY: `git diff --check` passed.
