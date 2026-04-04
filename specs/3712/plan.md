# Plan: Issue #3712 - Surface scaffold placeholder summary in tau-tui command palette preview

Status: Reviewed

## Goal
Expose unresolved scaffold placeholders directly in the command-palette preview
for parameterized commands while keeping simple command previews unchanged.

## Approach
1. Extend the selected-command preview in
   `crates/tau-tui/src/interactive/ui_overlays.rs` to derive placeholder tokens
   from the selected scaffold and render a `Placeholders:` line only when at
   least one token exists.
2. Keep the preview behavior additive so the existing section, scaffold, usage,
   alias, and summary lines remain unchanged.
3. Add focused RED/GREEN tests in
   `crates/tau-tui/src/interactive/app_gateway_tests.rs` for both the simple
   and parameterized command cases.

## Affected Modules
- `crates/tau-tui/src/interactive/ui_overlays.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `specs/3712/spec.md`
- `specs/3712/tasks.md`
- `specs/milestones/m335/index.md`

## Risks / Mitigations
- Risk: placeholder preview could duplicate or conflict with active placeholder
  feedback.
  Mitigation: restrict the new summary to pre-activation selected preview data
  and keep active-placeholder messaging untouched.
- Risk: simple command previews could become noisier.
  Mitigation: render the line only when placeholders are present.

## Interfaces / Contracts
- No external API changes.
- `render_command_palette` remains the single source of command-palette preview
  text.
