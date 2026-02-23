# Plan: Issue #3428 - Dashboard/TUI operator-flow convergence

## Approach
1. Add RED tests that assert missing parity fields in TUI shell output.
2. Add a cross-surface RED test in `tau-dashboard-ui` comparing dashboard HTML and TUI shell output for shared operator snapshot values.
3. Expand `tau-tui::OperatorShellFrame` and renderer to include parity fields and deterministic text markers.
4. Re-run targeted dashboard/TUI selectors and capture verification evidence.

## Implementation Targets
- `crates/tau-tui/src/lib.rs`
- `crates/tau-tui/src/main.rs` (if shell defaults/fixtures need parity field initialization)
- `crates/tau-dashboard-ui/Cargo.toml` (dev-dependency for cross-surface test)
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/milestones/m296/index.md`
- `specs/3428/spec.md`
- `specs/3428/plan.md`
- `specs/3428/tasks.md`

## Risks and Mitigations
- Risk: widening TUI frame struct can break existing tests/fixtures.
  - Mitigation: update fixture defaults and all explicit constructors in one patch; keep rendering markers deterministic.
- Risk: cross-surface tests become brittle on unrelated dashboard markup changes.
  - Mitigation: assert stable marker/value snippets only, not large HTML fragments.
- Risk: accidental scope creep into gateway/API contracts.
  - Mitigation: limit changes to presentation-layer structs/tests only.

## Interfaces / Contracts
- `tau-tui::OperatorShellFrame` render contract (`render_operator_shell_frame`).
- `tau-dashboard-ui` shell rendering contract (`render_tau_ops_dashboard_shell_with_context`).
- Cross-surface parity assertions on shared control-plane signal values.

## ADR
Not required (no architecture/dependency/protocol decision change).
