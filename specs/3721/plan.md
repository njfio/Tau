# Plan: Issue #3721 - Normalize slash-prefixed explicit command-palette submissions in tau-tui

## Goal
Make slash-prefixed typed command-palette submissions execute through the same
command and alias resolution path as the normal slash-command input flow.

## Approach
1. Add focused RED tests for:
   - a slash-prefixed typed search command
   - a slash-prefixed typed clipboard command with a role argument
2. Normalize only the first token of palette command execution so `/search
   apples` behaves like `search apples` and `/copy-last assistant` behaves like
   `copy-last assistant`.
3. Leave filtering, preview rendering, and placeholder logic unchanged.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`

## Risks / Mitigations
- Risk: broad slash stripping mutates arguments or typed payloads.
  Mitigation: normalize only the first token before command lookup.
- Risk: argument-carrying commands lose or mutate payload text after
  normalization.
  Mitigation: normalize only the first token and leave the remaining argument
  string untouched.

## Verification
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui red_spec_3721 -- --test-threads=1`
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui interactive::app_gateway_tests -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-tui/src/interactive/app_commands.rs crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `CARGO_INCREMENTAL=0 cargo clippy -p tau-tui --no-deps -- -D warnings`
- `CARGO_INCREMENTAL=0 cargo build -p tau-tui`
