# Tasks: Issue #3482 - M306 command-center last-action reason visibility

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Functional/Regression): add failing dashboard-ui tests for
   populated and fallback Last Action reason row contracts.
2. [x] T2 (RED, Integration): add failing gateway `/ops` assertion for Last
   Action reason row + data marker contracts.
3. [x] T3 (GREEN, Implementation): add `last_action_reason` snapshot field,
   mapping, and rendered row/marker contracts.
4. [x] T4 (VERIFY): run scoped dashboard-ui/gateway tests plus clippy/fmt and
   update spec/task status to `Implemented`.

## Tier Mapping
| Tier | Planned Coverage |
| --- | --- |
| Unit | N/A (render/snapshot contract scope) |
| Property | N/A |
| Contract/DbC | N/A |
| Snapshot | N/A |
| Functional | dashboard-ui reason-row render assertions |
| Conformance | C-01/C-02/C-03 mapped selector tests |
| Integration | gateway `/ops` render reason-row assertions |
| Fuzz | N/A |
| Mutation | N/A (non-critical UI contract slice) |
| Regression | fallback `none` reason row assertion |
| Performance | N/A |

## TDD Evidence
### RED
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3482 -- --nocapture`
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3482 -- --nocapture`
- Expected failures before implementation:
  - missing `data-last-action-reason` marker and `tau-ops-last-action-reason` row in dashboard-ui render.
  - missing reason marker row in gateway `/ops` render.

### GREEN
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3482 -- --nocapture` passed.
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3482 -- --nocapture` passed.

### REGRESSION
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3478 -- --nocapture` passed.
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3478 -- --nocapture` passed.
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3466 -- --nocapture` passed.
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3466 -- --nocapture` passed.
- `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-dashboard-ui -p tau-gateway --tests --no-deps -- -D warnings` passed.
- `cargo fmt --check` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A |  | Render/snapshot contract scope only |
| Property | N/A |  | No randomized invariant surface |
| Contract/DbC | N/A |  | No DbC annotations introduced |
| Snapshot | N/A |  | No snapshot suite used |
| Functional | ✅ | `functional_spec_3482_c01_last_action_section_exposes_reason_row_and_marker_contracts`; `functional_spec_3482_c01_collect_command_center_snapshot_maps_last_action_reason` |  |
| Conformance | ✅ | `functional_spec_3482_c01_*`, `regression_spec_3482_c02_*`, `integration_spec_3482_c03_*` |  |
| Integration | ✅ | gateway `/ops` integration reason-row selector |  |
| Fuzz | N/A |  | No parser/input-surface changes |
| Mutation | N/A |  | Non-critical UI contract slice |
| Regression | ✅ | `regression_spec_3482_c02_last_action_reason_row_defaults_to_none` |  |
| Performance | N/A |  | No hotspot/perf path changes |
