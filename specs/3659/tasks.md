# Tasks: Issue #3659 - Expose gateway mission inspection and TUI resume controls for checkpointed Ralph-loop missions

- [x] T1 (RED): add gateway regression coverage for mission inventory and
      mission detail endpoints backed by persisted mission state.
- [x] T2 (RED): add interactive TUI functional coverage proving mission list and
      resume commands surface operator-visible mission state.
- [x] T3 (RED): add regression coverage proving resumed TUI turns include both
      `mission_id` and `session_id` in gateway request metadata.
- [x] T4 (GREEN): add gateway mission list/detail endpoints and route wiring.
- [x] T5 (GREEN): add interactive gateway mission client helpers and resume
      command handling.
- [x] T6 (GREEN): surface the active mission in interactive TUI status output.
- [x] T7 (VERIFY): run targeted `tau-gateway` and `tau-tui` mission-control
      verification.

## Tier Mapping
- Regression: mission inventory/detail endpoints and resumed-turn metadata
- Functional: interactive mission list/detail/resume command flows
- Integration: gateway mission persistence remains consumable through the new
  operator API and reused by the TUI
