# Tasks 3807: Intake Rerun Command

- [x] T1 (RED): Add spec, plan, tasks, and failing `tau-unified` status-contract
  assertions for `rerun_command`.
- [x] T2 (GREEN): Persist `issue_body` in intake records with legacy default.
- [x] T3 (GREEN): Generate shell-quoted rerun command in `tau-unified intake`.
- [x] T4 (DOCS): Document the rerun command behavior and safety gate.
- [x] T5 (VERIFY): Run focused shell/Rust tests, clippy, fmt, shell syntax,
  oversized-file guard, roadmap sync, and diff hygiene.

## Evidence

- RED:
  `bash scripts/run/test-tau-unified.sh status_contract` failed before
  implementation because `tau-unified intake` did not print
  `rerun_command=none` for vague intake and had no concrete rerun command for
  needs-authority intake.
- GREEN:
  `bash scripts/run/test-tau-unified.sh status_contract` passed after adding
  shell-quoted rerun command generation.
- RUNTIME:
  `CARGO_TARGET_DIR=/tmp/rust_pi-3807-rerun-target cargo test -p tau-runtime spec_3807 -- --test-threads=1`
  passed 4 intake contract tests, including legacy `issue_body` default.
  `CARGO_TARGET_DIR=/tmp/rust_pi-3807-rerun-target cargo test -p tau-runtime spec_3796_c04 -- --test-threads=1`
  passed persisted no-authority intake coverage.
- STATIC:
  `cargo fmt --check`; `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh`;
  `CARGO_TARGET_DIR=/tmp/rust_pi-3807-rerun-target cargo clippy -p tau-runtime --lib --tests -- -D warnings`;
  `CARGO_TARGET_DIR=/tmp/rust_pi-3807-rerun-target cargo clippy -p tau-coding-agent --bin tau_autonomous_coding_job -- -D warnings`.
- GUARDS:
  `python3 .github/scripts/oversized_file_guard.py --repo-root . --default-threshold 4000 --exemptions-file tasks/policies/oversized-file-exemptions.json --policy-guide docs/guides/oversized-file-policy.md --json-output-file /tmp/rust_pi-3807-rerun-oversized-file-guard.json`
  passed with 453 checked files and 0 issues;
  `scripts/dev/roadmap-status-sync.sh --check --quiet`; `git diff --check`.
