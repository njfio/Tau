# Tasks: Issue #3527 - M317 deterministic TUI scenario-expansion depth gate

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance/Regression): add
   `scripts/verify/test-m317-tui-scenario-expansion-depth.sh` with required-step
   and report-schema assertions; run pre-implementation to capture failure.
2. [x] T2 (GREEN, Implementation): add
   `scripts/verify/m317-tui-scenario-expansion-depth.sh` with deterministic
   TUI scenario-expansion selector map and JSON report generation.
3. [x] T3 (GREEN, Contract): enforce verify-only fail-closed required-step
   inventory checks.
4. [x] T4 (GREEN, Docs): update README TUI gap links with M317 entrypoint.
5. [x] T5 (VERIFY): run scoped script and formatting checks; mark implemented.

## TDD Evidence
### RED
- `bash scripts/verify/test-m317-tui-scenario-expansion-depth.sh`
- Expected failure before implementation:
  - `error: verification script missing or not executable: .../scripts/verify/m317-tui-scenario-expansion-depth.sh`

### GREEN
- `bash scripts/verify/test-m317-tui-scenario-expansion-depth.sh` passed.
- `CARGO_TARGET_DIR=target-fast TAU_M317_CARGO_TARGET_DIR=target-fast bash scripts/verify/m317-tui-scenario-expansion-depth.sh` passed.

### REGRESSION
- `bash -n scripts/verify/m317-tui-scenario-expansion-depth.sh scripts/verify/test-m317-tui-scenario-expansion-depth.sh` passed.
- `cargo fmt --check` passed.
## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A |  | Script/docs scope; no Rust unit surface changed |
| Property | N/A |  | No randomized invariant surface added |
| Contract/DbC | N/A |  | No `contracts` annotations introduced |
| Snapshot | N/A |  | No snapshot suite used |
| Functional | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m317-tui-scenario-expansion-depth.sh` |  |
| Conformance | ✅ | `bash scripts/verify/test-m317-tui-scenario-expansion-depth.sh` |  |
| Integration | ✅ | M317 selector map executes deterministic M311 + demo/parser/shell-live contracts |  |
| Fuzz | N/A |  | No new parser/input surface introduced |
| Mutation | N/A |  | Verification-shell/docs scope, non-critical mutation gate |
| Regression | ✅ | M317 contract pass/fail/tamper verify-only fail-closed checks |  |
| Performance | N/A |  | No performance hotspot changed |
