# M300 - Program hardening wave (live TUI resilience + integration verification sync)

Status: Active

## Context
M300 hardens the connected operator program path by improving `tau-tui`
`shell-live` behavior when dashboard/training artifacts are missing or malformed,
and by syncing README capability boundaries with current verified conformance.

Primary sources:
- `README.md`
- `crates/tau-tui/src/lib.rs`
- `crates/tau-tui/src/main.rs`

## Issue Hierarchy
- Epic: #3456
- Story: #3457
- Task: #3458

## Scope
- Add deterministic parse diagnostics for `shell-live` artifact loading.
- Surface missing/malformed file signals in operator shell alerts/actions.
- Add explicit conformance tests for malformed and missing artifact paths.
- Refresh README capability-boundary language to match verified integration
  behavior and concrete remaining boundaries.

## Exit Criteria
- `specs/3458/spec.md` is implemented with AC mapping and conformance evidence.
- New `tau-tui` conformance tests for malformed/missing artifacts pass.
- `cargo fmt --check`, `cargo clippy -p tau-tui --tests --no-deps -- -D warnings`,
  and scoped `tau-tui` tests pass.
- README reflects verified capability status without stale/ambiguous claims.
