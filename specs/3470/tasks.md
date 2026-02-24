# Tasks: Issue #3470 - M303 deterministic auth-depth verification gate

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance/Regression): add `scripts/verify/test-m303-auth-workflow-depth.sh` with required-step and report-schema assertions, then run it before implementation to capture expected failure.
2. [x] T2 (GREEN, Implementation): add `scripts/verify/m303-auth-workflow-depth.sh` with deterministic provider/gateway auth-depth selectors and JSON report emission.
3. [x] T3 (GREEN, Contract): add verify-only/fail-closed report checks so missing required step markers fail the gate.
4. [x] T4 (GREEN, Docs): update README auth workflow execution references to include the M303 gate entrypoint.
5. [x] T5 (VERIFY): run targeted script tests plus scoped formatting/lint checks for touched shell/docs/spec files, then set spec/task status to `Implemented`.

## TDD Evidence
### RED
- `bash scripts/verify/test-m303-auth-workflow-depth.sh`
- Expected failure before implementation:
  - `error: verification script missing or not executable: .../scripts/verify/m303-auth-workflow-depth.sh`

### GREEN
- `bash scripts/verify/test-m303-auth-workflow-depth.sh` passed.
- `CARGO_TARGET_DIR=target-fast bash scripts/verify/m303-auth-workflow-depth.sh` passed.

### REGRESSION
- `bash -n scripts/verify/m303-auth-workflow-depth.sh scripts/verify/test-m303-auth-workflow-depth.sh` passed.
- `cargo fmt --check` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A |  | Shell/docs workflow scope; no Rust unit surface changed |
| Property | N/A |  | No randomized invariant surface in this slice |
| Contract/DbC | N/A |  | No DbC annotation surface introduced |
| Snapshot | N/A |  | No snapshot suite used |
| Functional | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m303-auth-workflow-depth.sh` |  |
| Conformance | ✅ | `bash scripts/verify/test-m303-auth-workflow-depth.sh` (required step IDs + report schema checks) |  |
| Integration | ✅ | M303 script step selectors invoke gateway + provider integration tests for lifecycle and edge paths |  |
| Fuzz | N/A |  | No parser/input-surface expansion in code paths |
| Mutation | N/A |  | Verification-shell/docs slice, non-critical mutation gate |
| Regression | ✅ | Test script verify-only tamper case asserts fail-closed missing-step behavior |  |
| Performance | N/A |  | No performance hotspot or benchmark path changed |
