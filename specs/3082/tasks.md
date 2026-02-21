# Tasks: Issue #3082 - ops memory-graph edge style relation contracts

1. [ ] T1 (RED): add failing `functional_spec_3082_*` UI tests for edge-style marker contracts.
2. [ ] T2 (RED): add failing `integration_spec_3082_*` gateway tests for relation-type edge-style mappings.
3. [ ] T3 (GREEN): implement deterministic relation-type edge-style marker rendering.
4. [ ] T4 (REGRESSION): rerun selected suites (`spec_3078`, `spec_3070`, `spec_3068`, `spec_3064`, `spec_3060`, `spec_2921`, `spec_2917`, `spec_2913`, `spec_2909`, `spec_2905`).
5. [ ] T5 (VERIFY): run `cargo fmt --check`, `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`, and scoped spec suites.
