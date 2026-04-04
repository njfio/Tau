# Plan: Issue #3677 - Raise `tau-tui` to a world-class operator REPL

Status: Implemented

## Approach
This story will be executed as a staged REPL upgrade. The first slice focuses
on runtime control and observability inside the existing
`crates/tau-tui/src/interactive` surfaces rather than through a shell rewrite.

The implementation should:
- add a typed gateway status fetcher for `/gateway/status`
- add operator commands `/status`, `/retry`, and `/detach`
- store a compact runtime snapshot in `StatusBar`
- surface that snapshot in the status/help rendering and system transcript
- keep the change TUI-local and avoid new gateway endpoints

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - command handlers, pending-turn retry behavior, mission detach behavior,
    runtime snapshot state updates
- `crates/tau-tui/src/interactive/app_commands.rs`
  - slash command routing for `/status`, `/retry`, `/detach`
- `crates/tau-tui/src/interactive/gateway_client.rs`
  - typed `/gateway/status` fetch helper
- `crates/tau-tui/src/interactive/status.rs`
  - runtime snapshot display model and formatting helpers
- `crates/tau-tui/src/interactive/ui_status.rs`
  - top-bar and help-line rendering of runtime snapshot / recovery hints
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/green functional tests for command flows and rendering

## Contracts
- `/gateway/status` is fetched with the same auth handling and request timeout
  strategy as other gateway TUI requests
- `/retry` reuses `latest_user_prompt()` and the existing `submit_prompt()`
  flow; it must not invent a new turn path
- `/detach` only clears active mission binding in the TUI config/state; it does
  not mutate server-side mission storage
- runtime snapshot rendering must degrade gracefully when parts of the gateway
  status payload are absent

## Risks
- The TUI is already under active change in the current worktree, so the first
  slice must stay narrow and avoid destabilizing existing gateway flows
- Gateway status payloads contain broad JSON; the TUI should deserialize only
  the fields needed for operator feedback
- `/retry` should avoid surprising behavior when no prior user prompt exists

## Verification Strategy
- Add failing tests first for `/status`, `/retry`, `/detach`, and status-bar
  rendering
- Re-run existing gateway-backed TUI mission tests to protect current behavior
- Build `tau-tui` after the scoped test pass
