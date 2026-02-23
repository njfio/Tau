# Plan: Issue #3414 - README examples marker regression

## Approach
1. Reproduce failing regression in `examples_assets` test target (RED evidence).
2. Update `README.md` to include required example path markers referenced by the test contract.
3. Reproduce `tau-tools` memory regression failures observed in full-workspace run and harden the shared test policy helper for deterministic offline behavior.
4. Re-run failing targets and workspace regression checks (GREEN).

## Affected Modules
- `README.md`
- `crates/tau-tools/src/tools/tests.rs`
- `crates/tau-coding-agent/tests/examples_assets.rs` (read-only contract reference)
- `specs/milestones/m293/index.md`
- `specs/3414/spec.md`
- `specs/3414/plan.md`
- `specs/3414/tasks.md`

## Risks / Mitigations
- Risk: README text drift from test contract.
  - Mitigation: add explicit marker strings exactly as test expects.
- Risk: memory tests remain flaky under parallel/full-workspace execution.
  - Mitigation: use deterministic local-hash embedding config in shared test policy helper.
- Risk: scope creep into unrelated gate fixes.
  - Mitigation: keep this change bounded to README marker restoration plus `tau-tools` memory test helper determinism.

## Interfaces / Contracts
- Contract source: `crates/tau-coding-agent/tests/examples_assets.rs` `regression_readme_example_paths_exist_on_disk`.
- Expected markers:
  - `./examples/starter/package.json`
  - `./examples/extensions`
  - `./examples/extensions/issue-assistant/extension.json`
  - `./examples/extensions/issue-assistant/payload.json`
  - `./examples/events`
  - `./examples/events-state.json`
- Determinism hardening target:
  - `crates/tau-tools/src/tools/tests.rs` `test_policy_with_memory`

## ADR
- Not required (docs-only correction).
