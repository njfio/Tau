# Tasks: Issue #3494 - M309 deterministic auth credential lifecycle depth gate

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance/Regression): add
   `scripts/verify/test-m309-auth-credential-lifecycle-depth.sh` with required
   step and report-schema assertions; run pre-implementation to capture failure.
2. [x] T2 (GREEN, Implementation): add
   `scripts/verify/m309-auth-credential-lifecycle-depth.sh` with deterministic
   auth selector map and JSON report generation.
3. [x] T3 (GREEN, Contract): enforce verify-only fail-closed required-step
   inventory checks.
4. [x] T4 (GREEN, Docs): update README auth gap links with M309 entrypoint.
5. [x] T5 (VERIFY): run scoped script and formatting checks; mark implemented.

## TDD Evidence
### RED
- `bash scripts/verify/test-m309-auth-credential-lifecycle-depth.sh`
- Expected failure before implementation:
  - `error: verification script missing or not executable: .../scripts/verify/m309-auth-credential-lifecycle-depth.sh`

### GREEN
- `bash scripts/verify/test-m309-auth-credential-lifecycle-depth.sh` passed.
- `CARGO_TARGET_DIR=target-fast bash scripts/verify/m309-auth-credential-lifecycle-depth.sh` passed.

### REGRESSION
- `bash -n scripts/verify/m309-auth-credential-lifecycle-depth.sh scripts/verify/test-m309-auth-credential-lifecycle-depth.sh` passed.
- `cargo fmt --check` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A |  | Script/docs scope; no Rust unit surface changed |
| Property | N/A |  | No randomized invariant surface added |
| Contract/DbC | N/A |  | No `contracts` annotations introduced |
| Snapshot | N/A |  | No snapshot suite used |
| Functional | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m309-auth-credential-lifecycle-depth.sh` |  |
| Conformance | ✅ | `bash scripts/verify/test-m309-auth-credential-lifecycle-depth.sh` |  |
| Integration | ✅ | M309 selector map executes deterministic provider/gateway/integration-auth/secret-resolution contracts |  |
| Fuzz | N/A |  | No new parser/input surface introduced |
| Mutation | N/A |  | Verification-shell/docs scope, non-critical mutation gate |
| Regression | ✅ | M309 contract pass/fail/tamper verify-only fail-closed checks |  |
| Performance | N/A |  | No performance hotspot changed |
