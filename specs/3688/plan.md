# Plan: Issue #3688 - Add inline argument scaffolding to `tau-tui` command palette

Status: Implemented

## Approach
Extend the command catalog with a scaffold template field and update palette
autocomplete to insert that template instead of only the command name when the
selected command is parameterized. Keep non-parameterized commands on the
existing simple path so the behavior remains predictable.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
  - extend command catalog metadata with scaffold templates
- `crates/tau-tui/src/interactive/app.rs`
  - update selected-command autocomplete behavior to use scaffolds
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN tests for parameterized, alias-driven, and non-parameterized
    autocomplete behavior

## Contracts
- Parameterized commands autocomplete to canonical templates like
  `resume <mission-id>`
- Alias-selected commands still autocomplete to the canonical template
- Non-parameterized commands continue to autocomplete to the command name

## Risks
- Scaffold templates can drift from actual command usage if metadata is not kept
  consistent
- Autocomplete should not unexpectedly insert leading `/` when the palette uses
  command-mode text
- History and execute-on-enter behavior should remain unchanged

## Verification Strategy
- Add failing tests first for scaffolding behavior
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
