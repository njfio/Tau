# Plan: Issue #3714 - Surface alias-match provenance in tau-tui command palette preview

## Goal
Add a small preview-only hint that tells the operator when the current
command-palette query matched the selected command through an alias token.

## Approach
1. Extend the command-palette matching/preview path with a lightweight helper
   that can tell whether the current trimmed query exactly equals one of the
   selected command's aliases.
2. Render a `Matched via alias: <alias>` preview line only when that helper
   returns a value for the selected command.
3. Keep canonical-name matches and broader metadata matches quiet so the preview
   does not gain noisy provenance for ordinary queries.

## Affected Modules
- `crates/tau-tui/src/interactive/ui_overlays.rs`
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`

## Risks / Mitigations
- Risk: provenance logic drifts from real matching behavior.
  Mitigation: derive the alias signal from the same selected command and current
  trimmed query used by the palette.
- Risk: preview lines crowd the overlay and regress older preview tests.
  Mitigation: add one short conditional line and verify the full
  `interactive::app_gateway_tests` suite.

## Interfaces / Contracts
- No new user commands or gateway/runtime contracts.
- Preview output gains one optional line: `Matched via alias: <alias>`.

## Verification
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui red_spec_3714 -- --test-threads=1`
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui interactive::app_gateway_tests -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-tui/src/interactive/app_gateway_tests.rs`
