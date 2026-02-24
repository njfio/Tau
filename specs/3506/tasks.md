# Tasks: Issue #3506 - M312 deterministic auth live-env validation depth gate

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance/Regression): add
   `scripts/verify/test-m296-live-auth-validation.sh` and capture failing
   evidence for missing deterministic contract behavior.
2. [x] T2 (RED, Conformance/Regression): add
   `scripts/verify/test-m312-auth-live-env-depth.sh` and run before M312 script
   implementation to capture expected missing-script failure.
3. [x] T3 (GREEN, Implementation): add mock-mode support in
   `scripts/verify/m296-live-auth-validation.sh`.
4. [x] T4 (GREEN, Implementation): add
   `scripts/verify/m312-auth-live-env-depth.sh` with deterministic selector map
   and report generation.
5. [x] T5 (GREEN, Docs): update README auth gap links with M312 entrypoint.
6. [x] T6 (VERIFY): run scoped script and formatting checks; mark implemented.

## TDD Evidence
### RED
- `bash scripts/verify/test-m296-live-auth-validation.sh`
- Expected failure before mock-mode implementation:
  - `assertion failed (mock mode pass exit code): expected '0' got '20'`
- `bash scripts/verify/test-m312-auth-live-env-depth.sh`
- Expected failure before implementation:
  - `error: verification script missing or not executable: .../scripts/verify/m312-auth-live-env-depth.sh`

### GREEN
- `bash scripts/verify/test-m296-live-auth-validation.sh` passed.
- `bash scripts/verify/test-m312-auth-live-env-depth.sh` passed.
- `CARGO_TARGET_DIR=target-fast bash scripts/verify/m312-auth-live-env-depth.sh` passed.

### REGRESSION
- `bash -n scripts/verify/m296-live-auth-validation.sh scripts/verify/test-m296-live-auth-validation.sh scripts/verify/m312-auth-live-env-depth.sh scripts/verify/test-m312-auth-live-env-depth.sh` passed.
- `cargo fmt --check` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A |  | Script/docs scope; no Rust unit surface changed |
| Property | N/A |  | No randomized invariant surface added |
| Contract/DbC | N/A |  | No `contracts` annotations introduced |
| Snapshot | N/A |  | No snapshot suite used |
| Functional | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m312-auth-live-env-depth.sh` |  |
| Conformance | ✅ | `bash scripts/verify/test-m296-live-auth-validation.sh`; `bash scripts/verify/test-m312-auth-live-env-depth.sh` |  |
| Integration | ✅ | M312 selector map executes live-auth contract + auth-depth + credential-lifecycle suites |  |
| Fuzz | N/A |  | No new parser/input surface introduced |
| Mutation | N/A |  | Verification-shell/docs scope, non-critical mutation gate |
| Regression | ✅ | M296 skip/mock fail-closed checks and M312 pass/fail/tamper verify-only fail-closed checks |  |
| Performance | N/A |  | No performance hotspot changed |
