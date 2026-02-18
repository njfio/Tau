# Tasks: Issue #2425 - G12 phase-2 SkipTool implementation and validation

## Ordered Tasks
1. T1 (RED): add C-01/C-02 tests in `tau-tools` for built-in registry and skip payload contract.
2. T2 (RED): add C-03 integration test in `tau-agent-core` proving run-loop termination after skip.
3. T3 (RED): add C-04/C-05 tests for skip extraction and output suppression in runtime/gateway.
4. T4 (GREEN): implement `SkipTool` and register built-in name + registration wiring.
5. T5 (GREEN): implement agent-core skip detection and loop short-circuit termination.
6. T6 (GREEN): wire collectors/renderers to suppress fallback output when skip marker is present.
7. T7 (REGRESSION): run existing `/tau skip` regression test C-06 unchanged.
8. T8 (VERIFY): run `cargo fmt --check`, scoped `clippy`, and targeted test commands.
9. T9 (CLOSE): update issue/PR AC mapping, RED/GREEN evidence, tier matrix, and milestone links.

## Tier Mapping
- Unit: C-01, C-04
- Functional: C-02
- Conformance: C-01..C-06
- Integration: C-03, C-05
- Regression: C-06
- Property: N/A (no parser/invariant randomness added in this slice)
- Contract/DbC: N/A (no new DbC annotations in this slice)
- Snapshot: N/A (no stable snapshot output added)
- Fuzz: N/A (no new untrusted parser surface beyond existing command parser coverage)
- Mutation: N/A for iterative dev loop (pre-PR gate can run scoped mutants if requested)
- Performance: N/A (no hot-path algorithm changes)
