# Tasks: Issue #3535 - M319 unified one-command runtime entrypoint

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance): add `scripts/run/test-tau-unified.sh` and capture
   pre-implementation failure for missing launcher script.
2. [x] T2 (GREEN, Implementation): add `scripts/run/tau-unified.sh` with
   `up/status/down/tui` lifecycle behavior and deterministic artifacts.
3. [x] T3 (GREEN, Contract): enforce fail-closed argument/lifecycle edge paths.
4. [x] T4 (GREEN, Docs): update README + operator deployment guide with
   one-command launcher flow.
5. [x] T5 (VERIFY): run scoped test/syntax/format checks and mark implemented.

## TDD Evidence
### RED
- `bash scripts/run/test-tau-unified.sh`
- Expected failure before implementation:
  - `error: launcher script missing or not executable: .../scripts/run/tau-unified.sh`

### GREEN
- `bash scripts/run/test-tau-unified.sh` passed.

### REGRESSION
- `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh` passed.
- `cargo fmt --check` passed.
## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | N/A |  | Script/docs scope; no Rust unit surface changed |
| Property | N/A |  | No randomized invariant surface added |
| Contract/DbC | N/A |  | No `contracts` annotations introduced |
| Snapshot | N/A |  | No snapshot suite used |
| Functional | ✅ | `bash scripts/run/test-tau-unified.sh` |  |
| Conformance | ✅ | Launcher contract assertions for lifecycle and fail-closed args |  |
| Integration | ✅ | Runner-hook integration simulating up/status/down lifecycle |  |
| Fuzz | N/A |  | No new parser/input surface introduced |
| Mutation | N/A |  | Script/docs scope, non-critical mutation gate |
| Regression | ✅ | Edge-path checks (stale pid, unknown command, down when stopped) |  |
| Performance | N/A |  | No performance hotspot changed |
