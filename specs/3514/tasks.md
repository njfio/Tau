# Tasks: Issue #3514 - M314 deterministic dashboard operator workflow depth gate

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance/Regression): add
   `scripts/verify/test-m314-dashboard-operator-workflow-depth.sh` with
   required-step and report-schema assertions; run pre-implementation to
   capture failure.
2. [x] T2 (GREEN, Implementation): add
   `scripts/verify/m314-dashboard-operator-workflow-depth.sh` with
   deterministic dashboard ops workflow selector map and JSON report
   generation.
3. [x] T3 (GREEN, Contract): enforce verify-only fail-closed required-step
   inventory checks.
4. [x] T4 (GREEN, Docs): update README dashboard gap links with M314
   entrypoint.
5. [x] T5 (VERIFY): run scoped script and formatting checks; mark implemented.

## TDD Evidence
### RED
- `bash scripts/verify/test-m314-dashboard-operator-workflow-depth.sh`
- Expected failure before implementation:
  - `error: verification script missing or not executable: .../scripts/verify/m314-dashboard-operator-workflow-depth.sh`

### GREEN
- `bash scripts/verify/test-m314-dashboard-operator-workflow-depth.sh` passed.
- `CARGO_TARGET_DIR=target-fast bash scripts/verify/m314-dashboard-operator-workflow-depth.sh` passed.

### REGRESSION
- `bash -n scripts/verify/m314-dashboard-operator-workflow-depth.sh scripts/verify/test-m314-dashboard-operator-workflow-depth.sh` passed.
- `cargo fmt --check` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A |  | Script/docs scope; no Rust unit surface changed |
| Property | N/A |  | No randomized invariant surface added |
| Contract/DbC | N/A |  | No `contracts` annotations introduced |
| Snapshot | N/A |  | No snapshot suite used |
| Functional | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m314-dashboard-operator-workflow-depth.sh` |  |
| Conformance | ✅ | `bash scripts/verify/test-m314-dashboard-operator-workflow-depth.sh` |  |
| Integration | ✅ | M314 selector map executes deterministic ops chat/session/lineage/memory-graph/tools workflow contracts |  |
| Fuzz | N/A |  | No new parser/input surface introduced |
| Mutation | N/A |  | Verification-shell/docs scope, non-critical mutation gate |
| Regression | ✅ | M314 contract pass/fail/tamper verify-only fail-closed checks |  |
| Performance | N/A |  | No performance hotspot changed |
