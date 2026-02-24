# Tasks: Issue #3458 - Harden `tau-tui` shell-live parsing and sync README boundaries

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Functional): add failing `tau-tui` tests for malformed JSON
   artifact diagnostics.
2. [x] T2 (RED, Regression): add failing `tau-tui` tests for malformed JSONL
   diagnostics and missing-file fallback stability.
3. [x] T3 (GREEN, Implementation): add deterministic read/parse diagnostic
   capture and integrate it into shell alerts/actions.
4. [x] T4 (GREEN, Docs): update README capability-boundary language/references
   to match verified current integration status.
5. [x] T5 (VERIFY): run scoped `tau-tui` tests, `cargo fmt --check`, and
   `cargo clippy -p tau-tui --tests --no-deps -- -D warnings`.
6. [x] T6 (VERIFY): update AC verification and tier matrix; set spec status to
   `Implemented`.

## TDD Evidence
### RED
- Command:
  `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui spec_c01_live_shell_frame_reports_malformed_json_artifact_diagnostics -- --nocapture`
- Result: failed as expected before summary implementation with assertion on
  missing `artifact diagnostics summary` line.

### GREEN
- Command:
  `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui spec_c01_live_shell_frame_reports_malformed_json_artifact_diagnostics -- --nocapture`
- Result: passed after implementing diagnostic summary aggregation.

### REGRESSION
- `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui -- --nocapture` passed.
- `cargo fmt --check` passed.
- `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-tui --tests --no-deps -- -D warnings` passed.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests | N/A Why |
| --- | --- | --- | --- |
| Unit | ✅ | `spec_c01_live_shell_frame_reports_malformed_json_artifact_diagnostics`, `spec_c02_live_shell_frame_reports_malformed_jsonl_artifact_diagnostics` |  |
| Property | N/A |  | No randomized invariant/parsing surface was introduced in this slice |
| Contract/DbC | N/A |  | No new `contracts`-based API annotations were added |
| Snapshot | N/A |  | No snapshot fixtures/output contracts were introduced |
| Functional | ✅ | `spec_c01_live_shell_frame_reports_malformed_json_artifact_diagnostics`, `spec_c02_live_shell_frame_reports_malformed_jsonl_artifact_diagnostics` |  |
| Conformance | ✅ | `spec_c01_live_shell_frame_reports_malformed_json_artifact_diagnostics`, `spec_c02_live_shell_frame_reports_malformed_jsonl_artifact_diagnostics`, `regression_live_shell_frame_handles_missing_artifacts_without_panicking` |  |
| Integration | N/A |  | Change is crate-local TUI frame construction (no cross-service composition path touched) |
| Fuzz | N/A |  | No new untrusted parser entrypoint was added beyond existing serde parsing paths |
| Mutation | N/A |  | Scope is operator diagnostics/reporting path; no critical algorithmic/security path targeted for mutation in this wave |
| Regression | ✅ | `regression_live_shell_frame_handles_missing_artifacts_without_panicking`, full `cargo test -p tau-tui` run |  |
| Performance | N/A |  | No performance-sensitive hot path requirement changed in this wave |
