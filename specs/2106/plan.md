# Plan #2106

Status: Implemented
Spec: specs/2106/spec.md

## Approach

1. Identify public helper functions/constants in the scoped first-wave files.
2. Add concise `///` comments to clarify intent/contract for each public API.
3. Add a regression shell script under `scripts/dev/` that asserts required
   rustdoc markers exist in each scoped file.
4. Run scoped compile/test commands for affected crates and capture evidence.

## Affected Modules

- `specs/milestones/m28/index.md`
- `specs/2106/spec.md`
- `specs/2106/plan.md`
- `specs/2106/tasks.md`
- `crates/tau-github-issues/src/issue_runtime_helpers.rs`
- `crates/tau-github-issues/src/issue_command_usage.rs`
- `crates/tau-ai/src/retry.rs`
- `crates/tau-runtime/src/slack_helpers_runtime.rs`
- `scripts/dev/test-split-module-rustdoc.sh`

## Risks and Mitigations

- Risk: doc comments become stale or incorrect.
  - Mitigation: keep comments concise and behavior-descriptive (not speculative).
- Risk: scope expands and slows delivery.
  - Mitigation: enforce strict first-wave file list in spec.
- Risk: regression script is too brittle.
  - Mitigation: assert stable markers tied to specific public API names.

## Interfaces and Contracts

- Regression script:
  `bash scripts/dev/test-split-module-rustdoc.sh`
- Compile checks:
  `cargo check -p tau-github-issues --target-dir target-fast`
  `cargo check -p tau-ai --target-dir target-fast`
  `cargo check -p tau-runtime --target-dir target-fast`
- Targeted tests:
  `cargo test -p tau-github-issues issue_runtime_helpers --target-dir target-fast`
  `cargo test -p tau-github-issues issue_command_usage --target-dir target-fast`
  `cargo test -p tau-ai retry --target-dir target-fast`
  `cargo test -p tau-runtime slack_helpers_runtime --target-dir target-fast`

## ADR References

- Not required.
