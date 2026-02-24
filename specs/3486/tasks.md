# Tasks: Issue #3486 - M307 deterministic C5 multi-channel orchestration depth gate

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance/Regression): add
   `scripts/verify/test-m307-multi-channel-orchestration-depth.sh` with
   required-step and report-schema assertions; run before implementation to
   capture expected failure.
2. [x] T2 (GREEN, Implementation): add
   `scripts/verify/m307-multi-channel-orchestration-depth.sh` with deterministic
   C5 step selectors and JSON report emission.
3. [x] T3 (GREEN, Contract): add verify-only fail-closed checks for required
   step IDs and report consistency.
4. [x] T4 (GREEN, Docs): update README verification links with M307 entrypoint.
5. [x] T5 (VERIFY): run scoped script + formatting/lint checks, then mark
   status `Implemented` with evidence and tier matrix.

## TDD Evidence
### RED
- `bash scripts/verify/test-m307-multi-channel-orchestration-depth.sh`
- Expected failure before implementation:
  - `error: verification script missing or not executable: .../scripts/verify/m307-multi-channel-orchestration-depth.sh`

### GREEN
- `bash scripts/verify/test-m307-multi-channel-orchestration-depth.sh` passed.
- `CARGO_TARGET_DIR=target-fast bash scripts/verify/m307-multi-channel-orchestration-depth.sh` passed.

### REGRESSION
- `bash -n scripts/verify/m307-multi-channel-orchestration-depth.sh scripts/verify/test-m307-multi-channel-orchestration-depth.sh` passed.
- `cargo fmt --check` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A |  | Script/docs scope; no Rust unit surface changed |
| Property | N/A |  | No randomized invariant surface added |
| Contract/DbC | N/A |  | No `contracts` annotations introduced |
| Snapshot | N/A |  | No snapshot suite used |
| Functional | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m307-multi-channel-orchestration-depth.sh` |  |
| Conformance | ✅ | `bash scripts/verify/test-m307-multi-channel-orchestration-depth.sh` (schema/step inventory/tamper checks) |  |
| Integration | ✅ | M307 selector map runs deterministic gateway + multi-channel integration selectors for C5-01..C5-08 |  |
| Fuzz | N/A |  | No new parser/input surface introduced |
| Mutation | N/A |  | Verification-shell/docs scope, non-critical mutation gate |
| Regression | ✅ | M307 script contract test fail/tamper/verify-only fail-closed paths |  |
| Performance | N/A |  | No performance hotspot changed |
