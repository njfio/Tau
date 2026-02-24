# M311 - TUI operator workflow depth verification wave

Status: Active

## Context
M311 deepens TUI verification coverage by adding one deterministic gate that
aggregates shell panel conformance, shell-live watch contracts, and
artifact-diagnostic fail-closed behavior into a single auditable report.

Primary sources:
- `crates/tau-tui/src/lib.rs`
- `crates/tau-tui/src/main.rs`
- `crates/tau-tui/tests/tui_demo_smoke.rs`
- `scripts/verify/m295-operator-maturity-wave.sh`

## Issue Hierarchy
- Epic: #3500
- Story: #3501
- Task: #3502

## Scope
- Add deterministic M311 TUI workflow verification script and report contract.
- Add script contract test with fail-closed required-step checks.
- Map TUI operator shell/shell-live contracts to executable selectors.
- Update README links with M311 entrypoint.

## Exit Criteria
- `specs/3502/spec.md` is `Implemented` with AC evidence.
- M311 script report includes all required TUI workflow step IDs.
- Contract test fails closed on missing required-step IDs.
- README includes M311 verification entrypoint.
