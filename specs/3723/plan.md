# Plan: Issue #3723 - Surface slash-prefixed typed command matches in tau-tui command palette

## Goal
Keep command matches visible for slash-prefixed typed palette inputs that
already name a known command but also include real argument payloads.

## Approach
1. Add focused RED coverage for:
   - `command_palette_matches("/search apples")`
   - rendered palette behavior for `/resume mission-42`
2. Teach palette matching to recognize a canonical first command token from a
   slash-prefixed typed input without discarding the rest of the payload.
3. Leave execution and Enter-preview semantics unchanged.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`

## Risks / Mitigations
- Risk: slash-aware matching becomes broader than intended and changes generic
  multi-token matching behavior.
  Mitigation: scope the new behavior to slash-prefixed typed inputs with a
  recognized first command token.
- Risk: matching diverges from existing canonical command resolution.
  Mitigation: reuse the same first-token normalization helpers already used for
  execution and preview.

## Verification
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui red_spec_3723 -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-tui/src/interactive/app_gateway_tests.rs`
