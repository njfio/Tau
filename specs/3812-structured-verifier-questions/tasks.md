# Tasks 3812: Structured Repo-Aware Verifier Questions

- [x] T1 (RED): Add tests for structured missing package/filter clarifying
  questions and persisted reload.
- [x] T2 (GREEN): Reuse repo-aware verifier missing-input constants.
- [x] T3 (GREEN): Append structured clarifying questions from the verifier plan.
- [x] T4 (DOCS): Document the new reason codes.
- [x] T5 (VERIFY): Run focused tests, runtime module tests, clippy, fmt,
  oversized-file guard, roadmap sync check, and diff hygiene.

## Evidence

- RED:
  `CARGO_TARGET_DIR=/tmp/rust_pi-3812-target cargo test -p tau-runtime spec_3812 -- --test-threads=1`
  failed before implementation because `repo_aware_test_filter` and
  `repo_aware_cargo_package` clarifying questions were not present.
- GREEN:
  `CARGO_TARGET_DIR=/tmp/rust_pi-3812-target cargo test -p tau-runtime spec_3812 -- --test-threads=1`
  passed 2 structured clarifying question tests.
- REGRESSION:
  `CARGO_TARGET_DIR=/tmp/rust_pi-3812-target cargo test -p tau-runtime spec_3807_c03 -- --test-threads=1`
  passed the ready-intake guard.
- RUNTIME:
  `CARGO_TARGET_DIR=/tmp/rust_pi-3812-target cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1`
  passed 30 filtered runtime tests.
- STATIC:
  `cargo fmt --check`; `CARGO_TARGET_DIR=/tmp/rust_pi-3812-target cargo clippy -p tau-runtime --lib --tests -- -D warnings`.
- GUARDS:
  `python3 .github/scripts/oversized_file_guard.py --repo-root . --default-threshold 4000 --exemptions-file tasks/policies/oversized-file-exemptions.json --policy-guide docs/guides/oversized-file-policy.md --json-output-file /tmp/rust_pi-3812-oversized-file-guard.json`
  passed with 453 checked files and 0 issues;
  `scripts/dev/roadmap-status-sync.sh --check --quiet`; `git diff --check`.
