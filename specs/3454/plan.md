# Plan: Issue #3454 - M299 close E2E scenario gaps M7/X9/D12

Status: Implemented

## Approach
1. Baseline/spec setup:
   - add milestone/issue artifacts for M299 and align AC/conformance mapping.
2. RED-first matrix tests:
   - add explicit M7/X9/D12 conformance matrix tests in
     `crates/tau-gateway/src/gateway_openresponses/tests.rs`.
3. GREEN implementation:
   - extend scoped test harness helpers only as needed for new matrices,
   - avoid production-path behavior changes unless RED reveals regressions.
4. Verification/closeout:
   - run targeted test commands, fmt, and clippy,
   - update spec/tasks with AC and tier evidence for PR merge.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/milestones/m299/index.md`
- `specs/3454/spec.md`
- `specs/3454/plan.md`
- `specs/3454/tasks.md`

## Risks / Mitigations
- Risk: flaky stream assertions in dashboard/cortex SSE paths.
  - Mitigation: assert deterministic event markers and bounded read loops.
- Risk: over-scoping into unrelated endpoint behavior.
  - Mitigation: keep changes in test layer, reuse existing helpers/constants.
- Risk: compile/latency cost in large gateway test module.
  - Mitigation: use scoped selectors and isolated target dir when needed.

## Interfaces / Contracts
- M7 contract: memory CRUD/search/graph routes preserve deterministic API shape.
- X9 contract: cortex chat/status/bulletin semantics remain stable and observable.
- D12 contract: dashboard live-data routes and stream emit expected contract
  markers/events.

## Execution Summary
1. Added M299 milestone/index and #3454 spec/plan/tasks artifacts, then moved
   issue #3454 to implementation status.
2. Added dedicated conformance matrices in
   `crates/tau-gateway/src/gateway_openresponses/tests.rs`:
   - `integration_spec_3454_c02_m7_memory_graph_and_persistence_matrix`
   - `integration_spec_3454_c03_x9_cortex_bulletin_and_cross_session_matrix`
   - `integration_spec_3454_c04_d12_dashboard_live_data_stream_matrix`
3. Extended `TauE2eHarness` with scoped helpers for memory, cortex, dashboard,
   stream reading, bulletin bootstrapping, and captured-LLM request inspection.
4. RED exposed a scope-filter logic gap in gateway memory search; fixed runtime
   filtering in `crates/tau-gateway/src/gateway_openresponses/memory_runtime.rs`
   using `MemoryScopeFilter::matches_scope` plus explicit memory-type gating.
5. Hardened M7 test assertions to kill escaped mutants and validate limit/scope/
   type contracts deterministically.

## Verification Notes
- `cargo fmt --check` passed.
- `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-gateway --tests --no-deps -- -D warnings` passed.
- Targeted conformance and regression suites passed (see tasks evidence matrix).
- Scoped mutation run:
  `cargo mutants -p tau-gateway --file crates/tau-gateway/src/gateway_openresponses/memory_runtime.rs -F handle_gateway_memory_read --timeout 120 --minimum-test-timeout 20 -- -p tau-gateway integration_spec_3454_c02_m7_memory_graph_and_persistence_matrix`
  final result: `6 mutants tested, 6 caught`.
