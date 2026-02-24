# Tasks: Issue #3454 - M299 close E2E scenario gaps M7/X9/D12

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance): add failing artifact/contract checks for M299
   milestone/index and issue spec package.
2. [x] T2 (GREEN, Spec/Docs): finalize M299 + #3454 artifact consistency and AC
   mapping.
3. [x] T3 (RED, Integration): add failing M7 matrix test for memory
   graph/persistence lifecycle contracts.
4. [x] T4 (GREEN, Implementation): implement test harness/assertion wiring to
   satisfy M7 matrix contracts.
5. [x] T5 (RED, Integration): add failing X9 matrix test for cortex
   chat/status/fallback/bulletin contracts.
6. [x] T6 (GREEN, Implementation): implement test harness/assertion wiring to
   satisfy X9 matrix contracts.
7. [x] T7 (RED, Functional/Integration): add failing D12 matrix test for
   dashboard live-data routes and stream contracts.
8. [x] T8 (GREEN, Implementation): implement test harness/assertion wiring to
   satisfy D12 matrix contracts.
9. [x] T9 (VERIFY, Regression): execute scoped fmt/clippy/tests and record
   RED/GREEN/REGRESSION evidence.
10. [x] T10 (VERIFY): set spec status to `Implemented` after all ACs pass.

## TDD Evidence
### RED
- Command: `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_3454_c02_m7_memory_graph_and_persistence_matrix -- --nocapture`
- Result: compile failed as expected with missing `TauE2eHarness` methods (`put_memory_entry`, `search_memory`, `new_with_bulletin`, dashboard stream helpers, etc.).

### GREEN
- Command: `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_3454_ -- --nocapture`
- Result: passed (3/3):
  - `integration_spec_3454_c02_m7_memory_graph_and_persistence_matrix`
  - `integration_spec_3454_c03_x9_cortex_bulletin_and_cross_session_matrix`
  - `integration_spec_3454_c04_d12_dashboard_live_data_stream_matrix`

### REGRESSION
- Command: `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_2667_c01_memory_entry_endpoints_support_crud_search_and_legacy_compatibility -- --nocapture`
- Result: passed.
- Command: `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_2717_c04_gateway_new_session_prompt_includes_latest_cortex_bulletin_snapshot -- --nocapture`
- Result: passed.
- Command: `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_dashboard_stream_supports_reconnect_reset_and_snapshot_updates -- --nocapture`
- Result: passed.
- Command: `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_2953_c01_c02_c04_cortex_chat_uses_llm_output_with_context_markers_and_stable_sse_order -- --nocapture`
- Result: passed.
- Command: `cargo fmt --check`
- Result: passed.
- Command: `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-gateway --tests --no-deps -- -D warnings`
- Result: passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | New harness helper methods exercised by integration matrices; existing module unit coverage remains green |  |
| Property | N/A |  | No new randomized invariant/parsing surface introduced |
| Contract/DbC | N/A |  | No new public API contract annotations added |
| Snapshot | N/A |  | No snapshot-based output contract introduced in this slice |
| Functional | ✅ | `integration_spec_3454_c04_d12_dashboard_live_data_stream_matrix` |  |
| Conformance | ✅ | `integration_spec_3454_c02_*`, `integration_spec_3454_c03_*`, `integration_spec_3454_c04_*`, artifact existence checks |  |
| Integration | ✅ | M7/X9 matrices use real gateway routes, session/memory/cortex subsystems, and scripted LLM behavior |  |
| Fuzz | N/A |  | No new untrusted parser/input surface in this change |
| Mutation | ✅ | `cargo mutants -p tau-gateway --file crates/tau-gateway/src/gateway_openresponses/memory_runtime.rs -F handle_gateway_memory_read --timeout 120 --minimum-test-timeout 20 -- -p tau-gateway integration_spec_3454_c02_m7_memory_graph_and_persistence_matrix` |  |
| Regression | ✅ | `integration_spec_2667_c01_*`, `integration_spec_2717_c04_*`, `integration_dashboard_stream_supports_reconnect_reset_and_snapshot_updates`, `integration_spec_2953_c01_c02_c04_*` |  |
| Performance | N/A |  | No production hot-path performance objective changed in this slice |

## Mutation Outcome
- Initial scoped runs exposed escaped mutants in scope/type filtering.
- Added stricter M7 mismatch probes and simplified runtime filter logic.
- Final scoped run result: `6 mutants tested, 6 caught`.
