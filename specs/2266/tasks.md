# Tasks #2266

Status: Completed
Spec: specs/2266/spec.md
Plan: specs/2266/plan.md

- T1 (tests first): add RED conformance tests C-01..C-04 with explicit `spec_c0x`
  naming, including path-with-space `ExecStart` regression.
- T2: implement minimal systemd `ExecStart` rendering fix for executable path
  safety.
- T3: run scoped verification (`cargo fmt --check`, `cargo clippy -p tau-ops -- -D warnings`,
  `cargo test -p tau-ops`) and confirm AC mapping.
- T4: update issue process log, open PR with RED/GREEN evidence, merge, and set
  spec/tasks statuses to Implemented/Completed.
