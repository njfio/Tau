# Tasks: Issue #3044 - KAMN core malformed-input and normalization hardening

## Ordered Tasks
1. [x] T1 (RED): add failing conformance tests for empty dotted-segment rejection in `network` and `subject`.
2. [x] T2 (RED/GREEN): add conformance tests for canonical normalization and deterministic key-method outputs.
3. [x] T3 (GREEN): implement minimal identifier validation guard for empty dotted segments.
4. [x] T4 (REGRESSION): rerun targeted and full `kamn-core` tests.
5. [x] T5 (VERIFY): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, and `cargo check -q`.

## Tier Mapping
- Unit: identifier validation and deterministic output tests in crate unit module
- Property: N/A (no randomized invariant requirement in this slice)
- Contract/DbC: N/A (no contracts annotations in this crate)
- Snapshot: N/A (no snapshot output surfaces)
- Functional: request validation and DID build behavior tests
- Conformance: C-01..C-05
- Integration: crate-level deterministic behavior coverage across method/rendering path
- Fuzz: N/A (no new parser surface added)
- Mutation: N/A (non-critical coverage slice; follow-up if designated critical)
- Regression: targeted + full crate re-run with malformed-input guard assertions
- Performance: N/A (no perf-path changes)
