# Plan: Issue #3720 - Reuse real Enter preview guidance in empty-match tau-tui command palette state

## Goal
Make the command-palette empty-match overlay display the same Enter-preview
guidance already used in matched states, so no-match typed commands and
no-match unresolved placeholders stay honest.

## Approach
1. Add focused RED tests for:
   - a no-match resolved typed command
   - a no-match unresolved-placeholder command
2. Replace the hardcoded no-match guidance string in
   `crates/tau-tui/src/interactive/ui_overlays.rs` with
   `command_palette_enter_preview(app)`.
3. Keep the existing `No matching commands` line intact.

## Affected Modules
- `crates/tau-tui/src/interactive/ui_overlays.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`

## Risks / Mitigations
- Risk: no-match matched-state behavior drifts from helper logic.
  Mitigation: route the no-match branch through the existing preview helper
  instead of duplicating strings.
- Risk: the overlay loses explicit no-match feedback.
  Mitigation: preserve the current `No matching commands` line and test for it.

## Verification
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui red_spec_3720 -- --test-threads=1`
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui interactive::app_gateway_tests -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-tui/src/interactive/ui_overlays.rs crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `CARGO_INCREMENTAL=0 cargo clippy -p tau-tui --no-deps -- -D warnings`
- `CARGO_INCREMENTAL=0 cargo build -p tau-tui`
