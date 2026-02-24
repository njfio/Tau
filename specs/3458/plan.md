# Plan: Issue #3458 - Harden `tau-tui` shell-live parsing and sync README boundaries

Status: Implemented

## Approach
1. Add RED tests in `tau-tui` for malformed JSON and malformed JSONL artifact
   inputs, plus missing-file fallback behavior.
2. Introduce file-level read/parse diagnostic capture in `tau-tui` live-state
   loading helpers.
3. Thread diagnostic summary into `OperatorShellFrame` alerts/actions so
   operators can distinguish missing vs malformed artifacts.
4. Update README capability-boundary language and references to verified scripts.
5. Run scoped verification and capture RED/GREEN/REGRESSION evidence.

## Affected Modules
- `crates/tau-tui/src/lib.rs`
- `README.md`
- `specs/milestones/m300/index.md`
- `specs/3458/spec.md`
- `specs/3458/plan.md`
- `specs/3458/tasks.md`

## Risks / Mitigations
- Risk: noisy diagnostics reduce operator signal quality.
  - Mitigation: cap/normalize diagnostic text and prioritize high-value alerts.
- Risk: regression in existing live-shell defaults.
  - Mitigation: preserve current fallback behavior and extend regression tests.
- Risk: docs drift from executable evidence.
  - Mitigation: tie README updates to existing verification script references.

## Interfaces / Contracts
- `OperatorShellFrame::from_dashboard_state_dir` remains stable API-wise but
  gains deterministic diagnostic signaling in rendered fields.
- File-read helpers keep fail-closed/non-panicking behavior while exposing
  parse/missing distinctions for operator observability.

## Execution Summary
1. Added conformance selectors for malformed JSON/JSONL diagnostics and expanded
   missing-artifact regression assertions in `crates/tau-tui/src/lib.rs` tests.
2. Implemented deterministic diagnostic capture for live artifact loading with
   explicit labels: `parse_failed`, `missing`, `jsonl_malformed`, `read_failed`.
3. Added diagnostic summary aggregation in shell actions:
   `artifact diagnostics summary: parse_failed=.. missing=.. jsonl_malformed=.. read_failed=..`.
4. Threaded diagnostics into alerts/actions while preserving non-panicking
   fallback behavior for missing/invalid state artifacts.
5. Updated README capability boundaries and execution-plan rows to align with
   verified integration scripts and hardened TUI behavior.

## Verification Notes
- RED selector (expected failure before summary implementation):
  `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui spec_c01_live_shell_frame_reports_malformed_json_artifact_diagnostics -- --nocapture`
  failed on missing diagnostic-summary assertion.
- GREEN selector:
  same command passed after implementing summary aggregation.
- Regression + gates:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui -- --nocapture`
  - `cargo fmt --check`
  - `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-tui --tests --no-deps -- -D warnings`
  all passed.
