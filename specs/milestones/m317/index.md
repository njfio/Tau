# M317 - TUI scenario expansion depth verification wave

Status: Active

## Context
M317 deepens TUI interaction verification by adding one deterministic gate that
aggregates M311 operator workflow-depth contracts plus demo-mode behavior,
parser edge-path contracts, and shell-live argument/error-path checks into a
single auditable report.

Primary sources:
- `scripts/verify/m311-tui-operator-workflow-depth.sh`
- `crates/tau-tui/tests/tui_demo_smoke.rs`
- `crates/tau-tui/src/main.rs`
- `docs/guides/quickstart.md`

## Issue Hierarchy
- Epic: #3524
- Story: #3525
- Task: #3527

## Scope
- Add deterministic M317 TUI scenario-expansion verification script and report.
- Add script contract test with fail-closed required-step checks.
- Map demo/parse/shell-live TUI scenario contracts to executable selectors.
- Update README links with M317 verification entrypoint.

## Exit Criteria
- `specs/3527/spec.md` is `Implemented` with AC evidence.
- M317 report includes all required TUI scenario-expansion step IDs.
- Contract test fails closed on missing required-step IDs.
- README TUI gap entry includes M317 verification entrypoint.
