# Plan: Issue #3715 - Surface scaffold placeholder-match provenance in tau-tui command palette preview

## Goal
Add a small preview-only hint that tells the operator when the current
command-palette query matched the selected command through scaffold placeholder
text like `<mission-id>` or `<query>`.

## Approach
1. Extend the command-palette preview helpers with a lightweight placeholder
   provenance helper that compares the current trimmed query against the
   selected command's scaffold placeholders.
2. Render a `Matched via placeholder: <...>` preview line only when the helper
   finds an exact normalized placeholder match for the selected command.
3. Keep canonical-name and non-placeholder metadata matches quiet so the preview
   remains specific and low-noise.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/ui_overlays.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`

## Risks / Mitigations
- Risk: placeholder provenance drifts from search normalization behavior.
  Mitigation: compare the typed query against normalized placeholder text using
  the same alphanumeric compaction used by palette matching.
- Risk: extra preview lines crowd the overlay and regress existing tests.
  Mitigation: add one short conditional line and verify the full
  `interactive::app_gateway_tests` suite.

## Interfaces / Contracts
- No new user commands or gateway/runtime contracts.
- Preview output gains one optional line:
  `Matched via placeholder: <placeholder>`.

## Verification
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui red_spec_3715 -- --test-threads=1`
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui interactive::app_gateway_tests -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-tui/src/interactive/app_gateway_tests.rs`
