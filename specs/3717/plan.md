# Plan: Issue #3717 - Surface summary-match provenance in tau-tui command palette preview

## Goal
Add a small preview-only hint that tells the operator when the current
command-palette query matched the selected command through its summary text.

## Approach
1. Extend the command-palette preview helpers with a small summary-provenance
   helper that compares the current trimmed query against the selected command's
   summary text.
2. Render a `Matched via summary: <token>` preview line only when the helper
   finds an exact summary-token match for the selected command.
3. Keep canonical-name and other metadata matches quiet so the preview remains
   specific and low-noise.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/ui_overlays.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`

## Risks / Mitigations
- Risk: summary provenance drifts from palette normalization rules.
  Mitigation: compare the trimmed query against normalized summary tokens while
  keeping the final signal preview-only.
- Risk: extra preview lines crowd the overlay and regress existing preview tests.
  Mitigation: add one short conditional line and verify the full
  `interactive::app_gateway_tests` suite.

## Interfaces / Contracts
- No new user commands or gateway/runtime contracts.
- Preview output gains one optional line:
  `Matched via summary: <token>`.

## Verification
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui red_spec_3717 -- --test-threads=1`
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui interactive::app_gateway_tests -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-tui/src/interactive/app_gateway_tests.rs`
