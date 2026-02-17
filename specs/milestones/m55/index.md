# M55 — Cargo-Fuzz Harness Baseline for Untrusted Inputs

Milestone: [GitHub milestone #55](https://github.com/njfio/Tau/milestone/55)

## Objective

Close the remaining fuzz-testing partial gap by adding true `cargo-fuzz`
harnesses for high-risk untrusted parser surfaces.

## Scope

- Add `cargo-fuzz` harness targets for:
  - runtime RPC raw parser;
  - runtime RPC NDJSON parser;
  - gateway websocket parser.
- Seed initial corpus files and document local run procedure.
- Keep deterministic fuzz-conformance tests as complementary regression checks.

## Out of Scope

- Broad fuzzing for every crate in this milestone.
- Replacing existing deterministic fuzz-conformance tests.

## Linked Hierarchy

- Epic: #2339
- Story: #2340
- Task: #2341
- Subtask: #2342
