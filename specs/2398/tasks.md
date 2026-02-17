# Tasks: Issue #2398 - Apply role model overrides at orchestrator dispatch time

## Ordered Tasks
1. T1 (tests first): add failing C-01..C-03 integration/functional tests in routed orchestrator
   test harness to assert dispatched request model ordering and inherit behavior.
2. T2 (tests first): add failing C-05 unit test in `tau-agent-core` for scoped model restoration.
3. T3: implement scoped model helper in `tau-agent-core`.
4. T4: wire optional model override through orchestrator runtime contract + coding-agent adapter.
5. T5: ensure C-04 regression stays green (existing default-routed parity coverage).
6. T6: run `cargo fmt --check`, `cargo clippy -- -D warnings`, scoped tests, and
   `cargo mutants --in-diff`; fix any escapes.
7. T7: update issue process logs/labels, open PR with AC mapping + tier matrix, merge when green.

## Tier Mapping
- Unit: C-05 scoped helper restore.
- Functional: C-03 mixed override/inherit no-leak behavior.
- Integration: C-01, C-02 routed dispatch model ordering/inherit.
- Regression: C-04 existing routed-default parity.
- Mutation: `cargo mutants --in-diff` over touched files.
