# Plan: Issue #3044 - KAMN core malformed-input and normalization hardening

## Approach
1. Add RED conformance tests for malformed dotted-segment rejection and canonical normalization behavior in `kamn-core`.
2. Implement the smallest runtime validation change required to reject empty dotted segments in identifiers.
3. Re-run targeted and full crate tests plus formatting/lint/check gates.

## Affected Paths
- `crates/kamn-core/src/lib.rs`
- `specs/milestones/m189/index.md`
- `specs/3044/spec.md`
- `specs/3044/plan.md`
- `specs/3044/tasks.md`

## Risks and Mitigations
- Risk: over-constraining identifiers and breaking valid callers.
  - Mitigation: only reject clearly malformed empty dotted segments (`..`, leading/trailing dot) while preserving existing allowed characters.
- Risk: test ambiguity on expected error text.
  - Mitigation: assert stable error substring dedicated to empty-segment validation.

## Interfaces / Contracts
- `build_browser_did_identity` rejects malformed dotted identifiers for both `network` and `subject`.
- Canonical normalization of valid identifiers remains lowercase + trimmed.
- Existing DID key/web rendering contracts remain unchanged for valid inputs.

## ADR
Not required (localized validation hardening within existing crate boundary).
