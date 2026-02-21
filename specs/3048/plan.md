# Plan: Issue #3048 - KAMN SDK coverage hardening

## Approach
1. Add RED conformance tests for malformed-input propagation with SDK error context, normalized deterministic equivalence, and nested report persistence.
2. Implement the smallest SDK runtime change required by RED failures (expected: add context on `kamn-core` initialization failures).
3. Re-run targeted and full crate tests plus formatting/lint/check gates.

## Affected Paths
- `crates/kamn-sdk/src/lib.rs`
- `specs/milestones/m190/index.md`
- `specs/3048/spec.md`
- `specs/3048/plan.md`
- `specs/3048/tasks.md`

## Risks and Mitigations
- Risk: over-constraining error string matching.
  - Mitigation: assert stable substrings for context and source detail instead of full message equality.
- Risk: filesystem test flakiness.
  - Mitigation: use isolated temporary directories and deterministic JSON assertions.

## Interfaces / Contracts
- `initialize_browser_did` error surface includes SDK context on underlying `kamn-core` validation failures.
- Deterministic identity output remains stable for normalized-equivalent inputs.
- `write_browser_did_init_report` persists valid JSON and keeps newline-terminated file contract.

## ADR
Not required (localized test/runtime hardening within existing SDK boundary).
