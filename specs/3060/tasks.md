# Tasks: Issue #3060 - ops memory delete confirmation contracts

1. [ ] T1 (RED): add failing `functional_spec_3060_*` UI tests for delete form,
   confirmation controls, and delete status markers on `/ops/memory`.
2. [ ] T2 (RED): add failing `integration_spec_3060_*` gateway tests for
   confirmed delete success and unconfirmed/missing-target safety behavior.
3. [ ] T3 (GREEN): implement delete operation handling and deterministic status
   marker rendering contracts.
4. [ ] T4 (REGRESSION): rerun selected memory-route specs
   (`spec_2905`, `spec_2909`, `spec_2913`, `spec_2917`, `spec_2921`).
5. [ ] T5 (VERIFY): run `cargo fmt --check`,
   `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`, and
   scoped spec suites.
