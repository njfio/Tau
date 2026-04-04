# Plan: Issue #3718 - Surface literal scaffold-token provenance in tau-tui command palette preview

## Goal
Add a preview-only provenance helper that explains when the selected
command-palette match was discovered through a literal scaffold argument token
such as `assistant` or `system`.

## Approach
1. Add focused RED tests that cover two literal scaffold-token matches and one
   canonical-query omission case.
2. Add a `command_palette_scaffold_token_preview` helper alongside the existing
   alias, placeholder, section, and summary provenance helpers.
3. Keep the helper narrow:
   - require a non-empty single-token query
   - inspect only the selected command's scaffold text
   - ignore placeholder tokens like `<mission-id>`
   - ignore canonical command-name matches and other existing provenance paths
4. Render the optional provenance line in the command-palette preview block
   without altering command ranking or execution semantics.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/ui_overlays.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`

## Risks / Mitigations
- Risk: scaffold-token provenance overlaps with placeholder or summary provenance.
  Mitigation: short-circuit when earlier provenance helpers already match.
- Risk: optional bracket syntax pollutes the displayed token.
  Mitigation: tokenize scaffold text on non-alphanumeric separators and emit the
  exact literal token only.

## Verification
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui red_spec_3718 -- --test-threads=1`
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui interactive::app_gateway_tests -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-tui/src/interactive/app_commands.rs crates/tau-tui/src/interactive/ui_overlays.rs crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `CARGO_INCREMENTAL=0 cargo clippy -p tau-tui --no-deps -- -D warnings`
- `CARGO_INCREMENTAL=0 cargo build -p tau-tui`
