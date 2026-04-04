# Plan: Issue #3711 - Surface section and scaffold details in tau-tui command palette preview

Status: Reviewed

## Goal
Make the command palette preview more actionable by showing the selected
command's section and the exact scaffold string that autocomplete will insert.

## Approach
1. Extend the selected-command preview rendering in
   `crates/tau-tui/src/interactive/ui_overlays.rs` to include `Section:` and
   `Scaffold:` lines alongside the existing usage/alias/summary details.
2. Keep preview rendering tied to the existing selected command so the new
   lines automatically follow navigation changes.
3. Add focused RED/GREEN tests in
   `crates/tau-tui/src/interactive/app_gateway_tests.rs` covering both the
   default selected command and a parameterized command at the end of the list.

## Affected Modules
- `crates/tau-tui/src/interactive/ui_overlays.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `specs/3711/spec.md`
- `specs/3711/tasks.md`
- `specs/milestones/m335/index.md`

## Risks / Mitigations
- Risk: preview could become visually noisy.
  Mitigation: add concise `Section:` and `Scaffold:` lines without changing the
  rest of the overlay layout.
- Risk: selection-tracking preview behavior could regress.
  Mitigation: reuse the existing selected command object and run the broader
  `interactive::app_gateway_tests` regression suite in green phase.

## Interfaces / Contracts
- No external API changes.
- `render_command_palette` remains the single source of command palette overlay
  preview text.
