# Tasks: Issue #2608 - Integration suite bootstrap under tests/integration

## Ordered Tasks
1. T1 (RED): add `tests/integration` package with failing conformance test skeleton for agent -> memory_write -> memory_search roundtrip.
2. T2 (GREEN): wire package into root workspace and implement deterministic queued mock LLM client + isolated temp memory-state harness.
3. T3 (GREEN): register real `MemoryWriteTool` and `MemorySearchTool` on `Agent` and make conformance assertions pass.
4. T4 (REFACTOR): clean helper routines for readability and future reuse without changing behavior.
5. T5 (VERIFY): run `cargo fmt --check`, scoped `cargo clippy -p tau-integration-tests -- -D warnings`, and `cargo test -p tau-integration-tests`.
6. T6 (CLOSE): update issue status/process log and spec status to `Implemented` after verification.

## Tier Mapping
- Unit: helper parsing/assertion utilities in integration package
- Property: N/A (no randomized invariant surface introduced)
- Contract/DbC: N/A (no new contracts crate annotations in this slice)
- Snapshot: N/A (no stable snapshot artifact required)
- Functional: C-04
- Conformance: C-02
- Integration: C-01, C-02
- Fuzz: N/A (no new untrusted parser entrypoint)
- Mutation: N/A for this bootstrap slice (non-critical-path harness setup)
- Regression: C-03
- Performance: N/A (no hotspot/throughput change)
