# Plan: Issue #3168 - kamn-core label boundary validation hardening

## Approach
1. Add spec-mapped RED tests C-01..C-04 in `crates/kamn-core/src/lib.rs`.
2. Run targeted tests to capture RED failure for boundary cases.
3. Implement minimal validation guard in `normalize_identifier` for label boundary enforcement.
4. Re-run targeted tests to GREEN and execute crate verification checks.

## Affected Modules
- `crates/kamn-core/src/lib.rs`
- `specs/milestones/m219/index.md`
- `specs/3168/spec.md`
- `specs/3168/plan.md`
- `specs/3168/tasks.md`

## Risks & Mitigations
- Risk: over-restricting valid legacy identifiers.
  - Mitigation: boundary rule only checks label start/end; preserves existing supported character set and canonical normalization.
- Risk: brittle error assertions.
  - Mitigation: assert on stable message fragments only.

## Interfaces / Contracts
- `build_browser_did_identity` error contract for malformed `network` and `subject` inputs.
- `normalize_identifier` canonical normalization for valid identifiers.

## ADR
No ADR required (single-module validation hardening).
