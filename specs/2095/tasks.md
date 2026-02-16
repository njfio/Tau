# Tasks #2095

Status: Implemented
Spec: specs/2095/spec.md
Plan: specs/2095/plan.md

## Ordered Tasks

- T1 (RED): carry RED evidence from subtask `#2097` marker failure before
  execution-domain module wiring.
- T2 (GREEN): consume merged task/subtask delivery from PRs `#2100` and `#2099`.
- T3 (VERIFY): run
  `bash scripts/dev/test-cli-args-domain-split.sh`,
  `cargo check -p tau-cli --lib --target-dir target-fast`,
  and `cargo test -p tau-coding-agent startup_preflight_and_policy --target-dir target-fast`.
- T4 (CLOSE): set story status done and close issue `#2095` with conformance evidence.
