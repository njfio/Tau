# Plan: Issue #3719 - Surface unresolved-placeholder Enter blocking in tau-tui command palette preview

## Goal
Make the command-palette Enter preview honest for typed commands that still
contain unresolved placeholders, while preserving the existing selected/typed
preview wording for runnable inputs.

## Approach
1. Add focused RED tests for two unresolved-placeholder typed commands and one
   supported resolved typed command that already retains palette context.
2. Update `command_palette_enter_preview` to inspect the resolved submission and
   branch before the current selected/typed wording:
   - if the submission is typed and still has unresolved placeholders, return a
     blocking preview string that names them
   - otherwise keep the current selected/typed preview strings
3. Reuse `unresolved_placeholders` so preview guidance and submit-time blocking
   stay aligned.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`

## Risks / Mitigations
- Risk: the preview wording diverges from real submit behavior.
  Mitigation: derive the preview from the same unresolved-placeholder helper
  used by `submit_command_palette`.
- Risk: selected-command flows accidentally switch wording.
  Mitigation: keep the new branch limited to typed submissions only.

## Verification
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui red_spec_3719 -- --test-threads=1`
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui interactive::app_gateway_tests -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-tui/src/interactive/app_commands.rs crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `CARGO_INCREMENTAL=0 cargo clippy -p tau-tui --no-deps -- -D warnings`
- `CARGO_INCREMENTAL=0 cargo build -p tau-tui`
