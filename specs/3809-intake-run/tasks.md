# Tasks 3809: Intake Run

- [x] T1 (RED): Add spec, plan, tasks, and failing wrapper contract for
  `tau-unified intake-run`.
- [x] T2 (RED): Add focused binary tests for intake-run validation and request
  construction.
- [x] T3 (GREEN): Implement `tau-autonomous-coding-job intake-run`.
- [x] T4 (GREEN): Implement `tau-unified intake-run` wrapper.
- [x] T5 (DOCS): Document the command and safety gates.
- [x] T6 (VERIFY): Run focused shell/Rust tests, clippy, fmt, shell syntax,
  oversized-file guard, roadmap sync, and diff hygiene.

## Evidence

- RED: `bash scripts/run/test-tau-unified.sh status_contract` failed with exit
  code 2 before `tau-unified intake-run` existed.
- GREEN: `bash scripts/run/test-tau-unified.sh status_contract`
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3809-target cargo test -p
  tau-coding-agent --bin tau_autonomous_coding_job spec_3809 --
  --test-threads=1`
- GREEN: `cargo fmt --check`
- GREEN: `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh`
- GREEN: `CARGO_TARGET_DIR=/tmp/rust_pi-3809-target cargo clippy -p
  tau-coding-agent --bin tau_autonomous_coding_job -- -D warnings`
- GREEN: `python3 .github/scripts/oversized_file_guard.py --repo-root .
  --default-threshold 4000 --exemptions-file
  tasks/policies/oversized-file-exemptions.json --policy-guide
  docs/guides/oversized-file-policy.md --json-output-file
  /tmp/rust_pi-3809-oversized-file-guard.json`
- GREEN: `scripts/dev/roadmap-status-sync.sh --check --quiet`
- GREEN: `git diff --check`
