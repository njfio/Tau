# Tasks: Issue #2602 - G4 phase-2 branch tool runtime orchestration + limits

## Ordered Tasks
1. T1 (RED): add C-01/C-02 tests proving branch tool results trigger isolated branch follow-up and return structured conclusion payload.
2. T2 (RED): add C-03/C-05/C-06 regression tests for branch-concurrency enforcement, limit >1 behavior, and slot-release behavior.
3. T3 (RED): add C-04 regression test for missing/invalid branch prompt follow-up failure handling.
4. T4 (GREEN): implement branch follow-up runtime path and memory-only tool filtering in `tau-agent-core`.
5. T5 (GREEN): add `AgentConfig.max_concurrent_branches_per_session` and wire branch concurrency guard.
6. T6 (VERIFY): run `cargo fmt --check`, scoped `clippy`, and scoped `cargo test -p tau-agent-core` with conformance mapping.
7. T7 (CLOSE): update roadmap checkbox status and issue process log/closure evidence.

## Tier Mapping
- Unit: config default + branch helper edge conditions
- Functional: C-02
- Conformance: C-01..C-06
- Integration: C-01
- Regression: C-03, C-04, C-05, C-06
- Property: N/A (no new randomized invariant surface in this slice)
- Contract/DbC: N/A (no DbC annotation additions in this slice)
- Snapshot: N/A (no stable snapshot artifact requirement)
- Fuzz: N/A (no new parser/untrusted binary surface)
- Mutation: Required on touched critical branch follow-up path before merge
- Performance: N/A (no throughput-critical algorithmic change)
