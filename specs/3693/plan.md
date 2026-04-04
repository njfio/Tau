# Plan: Issue #3693 - Add reverse placeholder cycling and active feedback to `tau-tui` command palette

Status: Reviewed

## Approach
Extend the command-palette placeholder state from `#3692` so the operator can
move backward across unresolved placeholders as well as forward, then expose the
currently active placeholder in the palette overlay. Reuse the existing
placeholder-span model rather than introducing a new parser or command AST.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - add reverse placeholder focus behavior on top of the existing placeholder
    tracking state
- `crates/tau-tui/src/interactive/app_commands.rs`
  - route reverse-tab to the previous placeholder span
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - render explicit active-placeholder feedback in the command palette
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for reverse cycling and active-placeholder feedback

## Contracts
- `Tab` continues to move to the next unresolved placeholder or autocomplete
  when no placeholder is available
- Reverse-tab moves to the previous unresolved placeholder when one exists
- Palette rendering shows the active placeholder token and its ordinal position
  when placeholder focus is active
- Unresolved placeholders still fail the submission guardrail from `#3690`

## Risks
- Reverse placeholder movement must not disturb the forward placeholder editing
  path from `#3692`
- Active placeholder feedback must stay coherent when placeholder spans are
  added, cleared, or replaced
- The palette UI should remain readable even when there are no command matches

## Verification Strategy
- Add failing tests first for reverse placeholder cycling and active-placeholder
  feedback
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
