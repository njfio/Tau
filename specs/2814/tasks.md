# Tasks: Issue #2814 - Command-center timeline chart and range SSR markers

## Ordered Tasks
1. [x] T1 (RED): add failing `/ops` integration tests for timeline chart metadata and range selector markers.
2. [x] T2 (GREEN): extend `tau-dashboard-ui` command-center snapshot/markup for timeline chart + range selector markers.
3. [x] T3 (GREEN): extend ops shell controls range query parsing and gateway mapping.
4. [x] T4 (REGRESSION): run phase-1A..1H regression suites.
5. [x] T5 (VERIFY): run fmt/clippy/tests/mutation/guardrails and set spec status to `Implemented`.

## Tier Mapping
- Unit: UI marker rendering + range parser behavior.
- Property: N/A.
- Contract/DbC: N/A.
- Snapshot: N/A.
- Functional: timeline chart/range marker assertions.
- Conformance: C-01..C-04.
- Integration: gateway `/ops` render with dashboard fixtures.
- Fuzz: N/A.
- Mutation: `cargo mutants --in-diff <diff-file> -p tau-gateway -p tau-dashboard-ui`.
- Regression: phase-1A..1H contract suites.
- Performance: N/A.

## Verification Evidence
- `cargo fmt --check` ✅
- `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings` ✅
- `cargo test -p tau-dashboard-ui functional_spec_2814 -- --test-threads=1` ✅
- `cargo test -p tau-gateway functional_spec_2814 -- --test-threads=1` ✅
- `cargo test -p tau-gateway functional_spec_{2786,2794,2798,2802,2806,2810} -- --test-threads=1` ✅
- `cargo test -p tau-dashboard-ui` ✅
- `cargo test -p tau-gateway` ✅
- `python3 .github/scripts/oversized_file_guard.py` ✅
- `cargo mutants --in-diff /tmp/mutants_2814.diff -p tau-gateway -p tau-dashboard-ui` ✅ (`17 tested, 11 caught, 6 unviable, 0 escaped`)
