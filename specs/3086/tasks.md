# Tasks: Issue #3086 - ops memory-graph node detail panel contracts

1. [x] T1 (RED): add failing `functional_spec_3086_*` UI tests for node selection/detail-panel markers.
2. [x] T2 (RED): add failing `integration_spec_3086_*` gateway tests for selected-node graph detail contracts.
3. [x] T3 (RED): add failing gateway regression for selected ops-harness lineage nodes that are not persisted memory records.
4. [x] T4 (GREEN): implement deterministic node selection/detail-href and graph detail panel markers for persisted and lineage nodes.
5. [x] T5 (REGRESSION): rerun selected suites (`spec_3086`, harness lineage graph render, and `spec_3064` detail-panel regressions).
6. [x] T6 (VERIFY): run `cargo fmt --package tau-gateway --check`, `cargo clippy -p tau-gateway -- -D warnings`, and scoped spec suites.
7. [x] T7 (GREEN): add selected graph-detail proof and connected relation rows to the Memory Graph detail panel.
