# Spec: Issue #3458 - Harden `tau-tui` shell-live parsing and sync README boundaries

Status: Implemented

## Problem Statement
`tau-tui shell-live` currently reads dashboard/training artifacts with silent
`Option` fallbacks. When files are malformed JSON/JSONL, operators do not get a
deterministic signal that parsing failed, which can hide state corruption during
live triage. README capability-boundary text also needs explicit alignment with
verified integration state from the current conformance wave.

## Scope
In scope:
- Add deterministic parse-diagnostic capture for `shell-live` artifact reads.
- Surface diagnostics in `OperatorShellFrame` alerts/actions without panics.
- Add conformance tests for malformed and missing artifact scenarios.
- Update README boundaries to reference verified current behavior.

Out of scope:
- Dashboard artifact schema/protocol changes.
- New route surfaces in gateway/dashboard services.
- Live third-party network credential validation.

## Acceptance Criteria
### AC-1 Shell-live emits deterministic diagnostics for malformed and missing files
Given a dashboard state directory with missing or malformed artifacts,
when `OperatorShellFrame::from_dashboard_state_dir` builds a live frame,
then alerts/actions include deterministic file-level diagnostics instead of
silently treating malformed inputs as absent.

### AC-2 Shell-live remains stable and useful under artifact faults
Given malformed/missing artifact files,
when `shell-live` rendering runs,
then no panic occurs and the frame still renders with actionable fallback state.

### AC-3 Conformance tests cover malformed and missing artifact flows
Given new conformance selectors in `tau-tui`,
when tests run,
then malformed JSON, malformed JSONL, and missing-file scenarios are asserted
with deterministic expectations mapped to AC-1/AC-2.

### AC-4 README capability boundaries are synced to verified integration status
Given current conformance and verification scripts,
when README is reviewed,
then boundary language is explicit, accurate, and tied to concrete verification
references instead of stale ambiguous wording.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | malformed `state.json` and `auth-status.json` | build live shell frame | diagnostics include parse-failed entries for both files |
| C-02 | AC-1 | Functional | malformed JSONL (`runtime-events.jsonl`/`actions-audit.jsonl`) | build live shell frame | diagnostics include malformed-jsonl entries with file names |
| C-03 | AC-2 | Regression | only partial/missing artifacts exist | build live shell frame | no panic; fallback heartbeat/auth rows/alerts remain renderable |
| C-04 | AC-3 | Conformance | new `tau-tui` tests | run scoped selectors | malformed + missing scenarios pass deterministically |
| C-05 | AC-4 | Docs/Conformance | README capability boundary section | review references | wording and references align with verified scripts/tests |

## Success Metrics / Observable Signals
- `tau-tui` live frame exposes file-level diagnostics for parse failures.
- New/updated malformed + missing artifact tests pass in scoped runs.
- `README.md` capability/boundary text matches conformance evidence and current
  operator workflow scripts.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui spec_c01_live_shell_frame_reports_malformed_json_artifact_diagnostics -- --nocapture` and `spec_c02_live_shell_frame_reports_malformed_jsonl_artifact_diagnostics` verify deterministic parse diagnostics (`parse_failed`, `jsonl_malformed`, summary counts). |
| AC-2 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui regression_live_shell_frame_handles_missing_artifacts_without_panicking -- --nocapture` confirms non-panicking fallback with missing-artifact diagnostics. |
| AC-3 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-tui -- --nocapture` passes full `tau-tui` conformance/regression suite including new malformed/missing selectors. |
| AC-4 | ✅ | `README.md` capability-boundary and current-gap rows were updated to reference verified scripts (`scripts/verify/m295-operator-maturity-wave.sh`, `scripts/verify/m296-ga-readiness-gate.sh`, `scripts/verify/m296-live-auth-validation.sh`) and the hardened `shell-live` behavior. |
