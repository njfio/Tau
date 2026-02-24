# M304 - TUI live watch interaction depth wave

Status: Active

## Context
`tau-tui shell-live` currently renders one snapshot from dashboard artifacts.
Operators need deterministic refresh-mode behavior to monitor changing state
without rerunning the command manually.

Primary sources:
- `crates/tau-tui/src/main.rs`
- `crates/tau-tui/src/lib.rs`
- `README.md`

## Issue Hierarchy
- Epic: #3472
- Story: #3473
- Task: #3474

## Scope
- Add live watch mode controls to `tau-tui shell-live`:
  - `--watch`
  - `--iterations`
  - `--interval-ms`
- Emit deterministic watch-cycle output markers for operator workflows.
- Add parser/output contract tests for watch mode behavior.
- Update README command examples with watch mode usage.

## Exit Criteria
- `specs/3474/spec.md` status is `Implemented` with AC evidence.
- `tau-tui` watch mode flags parse and validate deterministically.
- Watch output contains deterministic cycle markers.
- README includes watch-mode command reference.
