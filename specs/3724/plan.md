# Plan: Issue #3724 - Canonicalize slash-prefixed alias typed Enter guidance in tau-tui command palette

## Goal
Make typed Enter-preview guidance for slash-prefixed alias submissions reflect
the canonical command that Enter will actually execute.

## Approach
1. Add focused RED coverage for:
   - `/rs mission-42`
   - `/mi mission-42`
2. Update the typed Enter-preview canonicalization helper so the first token is
   normalized and resolved through `find_command_palette_command(...)` before
   the preview string is rendered.
3. Keep matching, execution, and non-slash preview behavior unchanged.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`

## Risks / Mitigations
- Risk: preview canonicalization diverges from the real execution path.
  Mitigation: reuse the same first-token normalization plus command lookup shape
  that `execute_command(...)` already uses.
- Risk: typed preview guidance for canonical slash-prefixed commands regresses.
  Mitigation: keep the change limited to first-token alias resolution and retain
  existing slash-prefixed canonical regression coverage.

## Verification
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui red_spec_3724 -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-tui/src/interactive/app_gateway_tests.rs`
