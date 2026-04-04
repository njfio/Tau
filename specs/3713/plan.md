# Plan: Issue #3713 - Surface Enter execution target in tau-tui command palette preview

Status: Reviewed

## Goal
Make the command-palette preview explicit about what pressing `Enter` will run
in the matching state, whether that is the selected command token or the exact
typed command string.

## Approach
1. Add a small preview-time helper that mirrors current command-palette
   submission semantics without changing execution behavior.
2. Extend `render_command_palette` in
   `crates/tau-tui/src/interactive/ui_overlays.rs` to render an
   `Enter runs ...` line derived from that helper.
3. Add focused RED/GREEN tests in
   `crates/tau-tui/src/interactive/app_gateway_tests.rs` for both the default
   selected-command path and an explicit typed-command path.

## Affected Modules
- `crates/tau-tui/src/interactive/ui_overlays.rs`
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `specs/3713/spec.md`
- `specs/3713/tasks.md`
- `specs/milestones/m335/index.md`

## Risks / Mitigations
- Risk: preview guidance could drift from actual submission behavior.
  Mitigation: reuse the same resolution rules or a thin shared helper rather
  than duplicating different logic in the UI layer.
- Risk: preview could become noisy.
  Mitigation: replace the generic matching-state hint with a concrete execution
  line instead of adding another vague instruction line.

## Interfaces / Contracts
- No external API changes.
- Command execution behavior remains unchanged; only preview text is updated.
