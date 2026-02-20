# Tasks: Issue #2810 - Command-center control-mode/action SSR markers

## Ordered Tasks
1. [x] T1 (RED): add failing `/ops` shell integration tests for control-mode/action/last-action marker contracts.
2. [x] T2 (GREEN): extend `tau-dashboard-ui` command-center context/markup for control/action markers.
3. [x] T3 (GREEN): map gateway dashboard snapshot control data into shell context.
4. [x] T4 (REGRESSION): run phase-1A..1G regression suites.
5. [x] T5 (VERIFY): run fmt/clippy/tests/mutation/guardrails and set spec status to `Implemented`.

## Tier Mapping
- Unit: ui marker rendering + control snapshot mapping tests.
- Property: N/A.
- Contract/DbC: N/A.
- Snapshot: N/A.
- Functional: `/ops` control marker contract assertions.
- Conformance: C-01..C-04.
- Integration: gateway `/ops` render with control snapshot fixtures.
- Fuzz: N/A.
- Mutation: `cargo mutants --in-diff /tmp/mutants_2810.diff -p tau-gateway -p tau-dashboard-ui` (6 tested, 6 caught, 0 escaped).
- Regression: phase-1A..1G contract suites.
- Performance: N/A.
