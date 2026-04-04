# Plan: Issue #3722 - Canonicalize slash-prefixed typed Enter preview guidance in tau-tui command palette

## Goal
Make typed Enter-preview guidance reflect the actual command target that will
run after the `#3721` execution normalization path.

## Approach
1. Add focused RED tests for slash-prefixed typed preview guidance on:
   - `/search apples`
   - `/copy-last assistant`
2. Introduce a small shared normalization helper for typed preview rendering so
   only the first command token loses a single leading slash.
3. Leave command execution, matching, and placeholder blocking behavior
   unchanged.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`

## Risks / Mitigations
- Risk: preview canonicalization drifts from execution canonicalization.
  Mitigation: reuse the same first-token normalization rule instead of adding a
  preview-only variant with broader behavior.
- Risk: unprefixed typed preview guidance regresses.
  Mitigation: keep the change scoped to a single leading slash on the first
  token and rely on existing unprefixed preview coverage.

## Verification
- `CARGO_INCREMENTAL=0 cargo test -p tau-tui red_spec_3722 -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-tui/src/interactive/app_gateway_tests.rs`
