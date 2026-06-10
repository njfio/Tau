# Tasks 3810: Repo-Aware Code Verifier Planning

- [x] T1 (RED): Add failing tests for repo-aware Rust verifier derivation,
  missing-filter blocking, and missing-package blocking.
- [x] T2 (GREEN): Discover repo packages with `cargo metadata` and derive only
  concrete `cargo test -p <package> <filter>` commands.
- [x] T3 (GREEN): Persist the derived command in the intake verifier plan and
  feed it into `issue-to-merge`.
- [x] T4 (DOCS): Document the package/test-filter boundary.
- [x] T5 (VERIFY): Run focused tests, runtime module tests, clippy, fmt, and
  diff hygiene.
- [x] T6 (CI): Keep the touched runtime surface under the oversized-file guard
  by splitting verifier derivation and focused tests into submodules.

## Evidence

- RED: `CARGO_TARGET_DIR=/tmp/rust_pi-3810-target cargo test -p tau-runtime spec_3810 -- --test-threads=1` failed before implementation with `Blocked` instead of `PrReady`.
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3810-target cargo test -p tau-runtime spec_3810 -- --test-threads=1` passed.
- VERIFY: `cargo fmt --check` passed.
- VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3810-target cargo test -p tau-runtime spec_3810 -- --test-threads=1` passed: 3 passed.
- VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3810-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1` passed: 26 passed.
- VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3810-target cargo clippy -p tau-runtime --lib --tests -- -D warnings` passed.
- VERIFY: `git diff --check` passed.
- VERIFY: `python3 .github/scripts/oversized_file_guard.py --repo-root . --default-threshold 4000 --exemptions-file tasks/policies/oversized-file-exemptions.json --policy-guide docs/guides/oversized-file-policy.md --json-output-file /tmp/rust_pi-3810-oversized-file-guard.json` passed.
