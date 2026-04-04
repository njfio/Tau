# Plan: Issue #3686 - Add selected-command preview details to `tau-tui` command palette

Status: Implemented

## Approach
Extend `CommandPaletteCommand` with compact usage/help metadata and render a
selected-command preview block inside the command palette overlay. Reuse the
existing selection and paging logic so preview content always follows the
selected result rather than introducing a separate focus model.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
  - extend command catalog metadata with preview-oriented fields
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - render selected-command preview details inside the palette
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN tests for preview rendering and preview updates across selection

## Contracts
- Palette render shows the selected command’s description, aliases or usage,
  and at least one invocation hint
- The preview updates when selection changes via arrow keys or paging
- Empty-query and paged browsing still show the correct selected preview

## Risks
- The overlay can get visually crowded if preview text is too long
- Preview content must not break older tests that already assert on palette text
- Metadata should stay consistent with the actual command behavior

## Verification Strategy
- Add failing tests first for preview render and selection-tracking behavior
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
