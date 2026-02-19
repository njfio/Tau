# Tasks: Issue #2694 - PRD gateway jobs list and cancel endpoints

## Ordered Tasks
1. [x] T1 (RED): add failing integration/regression tests for C-01..C-05.
2. [x] T2 (GREEN): add runtime session-list helper and gateway jobs routes/status discovery metadata.
3. [x] T3 (GREEN): implement jobs list/cancel handlers with deterministic response contracts.
4. [x] T4 (REGRESSION): verify unknown job `404` and unauthorized fail-closed behavior.
5. [x] T5 (VERIFY): run scoped fmt/clippy/targeted tests and capture C-06 evidence.

## Tier Mapping
- Unit: endpoint helper behavior coverage.
- Property: N/A (no randomized invariant algorithm introduced).
- Contract/DbC: N/A (contracts macros not introduced in touched modules).
- Snapshot: N/A (explicit field assertions used).
- Functional: C-01, C-02.
- Conformance: C-01..C-06.
- Integration: C-01, C-02, C-05.
- Fuzz: N/A (no new parser/codec boundary requiring fuzz harness in this bounded slice).
- Mutation: N/A (bounded additive endpoint slice).
- Regression: C-03, C-04, C-05.
- Performance: N/A (no hotspot/perf budget contract changed).
