# Tasks: Issue #3070 - ops memory-graph node size importance contracts

1. [ ] T1 (RED): add failing `functional_spec_3070_*` UI tests for node-size marker contracts.
2. [ ] T2 (RED): add failing `integration_spec_3070_*` gateway tests for low/high importance node-size derivation.
3. [ ] T3 (GREEN): implement deterministic size-bucket/value marker rendering from normalized importance.
4. [ ] T4 (REGRESSION): rerun selected suites (`spec_3068`, `spec_3064`, `spec_3060`, `spec_2921`, `spec_2917`, `spec_2913`, `spec_2909`, `spec_2905`).
5. [ ] T5 (VERIFY): run `cargo fmt --check`, `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`, and scoped spec suites.
