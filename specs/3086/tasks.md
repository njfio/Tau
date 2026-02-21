# Tasks: Issue #3086 - ops memory-graph node detail panel contracts

1. [ ] T1 (RED): add failing `functional_spec_3086_*` UI tests for node selection/detail-panel markers.
2. [ ] T2 (RED): add failing `integration_spec_3086_*` gateway tests for selected-node graph detail contracts.
3. [ ] T3 (GREEN): implement deterministic node selection/detail-href and graph detail panel markers.
4. [ ] T4 (REGRESSION): rerun selected suites (`spec_3082`, `spec_3078`, `spec_3070`, `spec_3068`, `spec_3064`, `spec_3060`, `spec_2921`, `spec_2917`, `spec_2913`, `spec_2909`, `spec_2905`).
5. [ ] T5 (VERIFY): run `cargo fmt --check`, `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`, and scoped spec suites.
