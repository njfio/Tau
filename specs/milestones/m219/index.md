# M219 - kamn-core Identifier Label Boundary Hardening

Status: In Progress

## Context
`kamn-core` identifier parsing accepts labels that start or end with `-`/`_`, which weakens boundary validation for DID identity/auth inputs.

## Scope
- Enforce label boundary validation in `kamn-core` identifiers.
- Add conformance tests for malformed leading/trailing boundary markers.
- Keep canonical normalization and deterministic DID rendering behavior unchanged for valid inputs.

## Linked Issues
- Epic: #3166
- Story: #3167
- Task: #3168

## Success Signals
- `cargo test -p kamn-core spec_3168 -- --test-threads=1`
- `cargo test -p kamn-core`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
