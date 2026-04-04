# Plan: Issue #3692 - Add placeholder-aware editing to `tau-tui` command palette scaffolds

Status: Implemented

## Approach
Extend the command-palette input model so scaffolded commands can track an
active placeholder span and a cursor location. Reuse `Tab` to move focus into
the next unresolved placeholder, then make typing and backspace operate on the
active placeholder token as a unit instead of appending to the end of the raw
string.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - track command-palette cursor and active placeholder span
  - update autocomplete/history flows to preserve consistent palette state
- `crates/tau-tui/src/interactive/app_commands.rs`
  - route `Tab` between autocomplete and placeholder-jump behavior
  - keep unresolved-placeholder submission guardrails intact
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - render the command-palette cursor at the active placeholder position
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for placeholder jump, replacement, and clearing behavior

## Contracts
- A scaffolded command keeps its existing string content until the operator
  activates a placeholder span
- Placeholder-focused typing replaces that span instead of appending after it
- Placeholder-focused backspace clears the span in one step
- Unresolved placeholders still fail the submission guardrail from `#3690`

## Risks
- Palette state must stay coherent across autocomplete, history recall, and
  placeholder editing without breaking earlier M335 behavior
- Cursor rendering must not drift when the placeholder span is cleared or
  replaced
- `Tab` should remain useful for autocomplete when no active scaffold
  placeholder exists

## Verification Strategy
- Add failing tests first for placeholder jump, replacement, and clearing
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
