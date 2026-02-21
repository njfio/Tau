# M190 - KAMN SDK Coverage Hardening

## Context
`kamn-sdk` provides the browser DID initialization/reporting surface consumed by higher-level runtimes, but conformance coverage for malformed-input propagation and persisted report contracts remains thin.

## Scope
- Add conformance-focused tests for malformed-input error propagation and deterministic normalized outputs.
- Harden SDK error context where RED tests identify ambiguous failures.
- Keep changes scoped to `crates/kamn-sdk` and issue-linked spec artifacts.

## Linked Issues
- Epic: #3046
- Story: #3047
- Task: #3048
