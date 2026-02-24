# Tasks: Issue #3478 - M305 command-center last-action detail rows

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Functional/Regression): add failing dashboard-ui tests for
   Last Action readable row rendering and fallback value contracts.
2. [x] T2 (RED, Integration): add failing gateway `/ops` assertion for Last
   Action readable row marker contracts.
3. [x] T3 (GREEN, Implementation): render Last Action detail rows in
   `tau-dashboard-ui` while preserving existing data-attribute markers.
4. [x] T4 (VERIFY): run scoped dashboard-ui + gateway tests and formatting
   checks, then update spec/task status to `Implemented`.

## Tier Mapping
| Tier | Planned Coverage |
| --- | --- |
| Unit | N/A (UI render contract slice) |
| Property | N/A (no randomized invariant surface) |
| Contract/DbC | N/A |
| Snapshot | N/A |
| Functional | dashboard-ui Last Action row rendering assertions |
| Conformance | C-01/C-02/C-03 selector mapping |
| Integration | gateway `/ops` rendered contract assertion |
| Fuzz | N/A |
| Mutation | N/A (non-critical UI rendering slice) |
| Regression | default/no-action fallback row assertions |
| Performance | N/A |

## TDD Evidence
### RED
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3478 -- --nocapture`
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3478 -- --nocapture`
- Expected failures before implementation:
  - missing `tau-ops-last-action-*` readable row elements in dashboard-ui render.
  - missing readable row markers in gateway `/ops` render.

### GREEN
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3478 -- --nocapture` passed.
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3478 -- --nocapture` passed.

### REGRESSION
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3466 -- --nocapture` passed.
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3466 -- --nocapture` passed.
- `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-dashboard-ui -p tau-gateway --tests --no-deps -- -D warnings` passed.
- `cargo fmt --check` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A |  | UI contract rendering slice; no standalone unit logic added |
| Property | N/A |  | No randomized invariant surface |
| Contract/DbC | N/A |  | No DbC annotations introduced |
| Snapshot | N/A |  | No snapshot suite used |
| Functional | ✅ | `functional_spec_3478_c01_last_action_section_renders_readable_detail_rows` |  |
| Conformance | ✅ | `functional_spec_3478_c01_*`, `regression_spec_3478_c02_*`, `integration_spec_3478_c03_*` |  |
| Integration | ✅ | gateway `/ops` integration selector for readable rows |  |
| Fuzz | N/A |  | No parser/input-surface expansion |
| Mutation | N/A |  | Non-critical UI rendering slice |
| Regression | ✅ | `regression_spec_3478_c02_last_action_section_defaults_to_fallback_rows` |  |
| Performance | N/A |  | No hotspot/perf path changes |
