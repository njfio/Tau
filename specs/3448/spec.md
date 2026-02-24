# Spec: Issue #3448 - M298 wave-1 E2E harness and ops dashboard conformance slice

Status: Implemented

## Problem Statement
Tau has strong component-level tests but lacks a bounded, unified first-slice execution contract that proves gateway/runtime E2E behavior and ops dashboard live controls remain aligned under one deterministic verification flow.

## Scope
In scope:
- Establish wave-1 E2E harness foundation and first integrated scenarios from the E2E PRD.
- Define and verify dashboard live data/control conformance checkpoints from the ops dashboard PRD.
- Wire verification evidence into existing script/test cadence with deterministic outputs.

Out of scope:
- Full implementation of all 15 scenario groups in `specs/tau-e2e-testing-prd.md`.
- Full UI redesign/replatform of dashboard views.
- Live third-party provider dependency in required CI paths.

## Acceptance Criteria
### AC-1 Milestone/issue artifacts anchor the wave-1 execution contract
Given M298 issue hierarchy,
when the issue is in implementation,
then `specs/milestones/m298/index.md` and `specs/3448/{spec,plan,tasks}.md` exist, are coherent, and map ACs to conformance cases.

### AC-2 E2E harness wave-1 scenarios execute deterministically
Given a scripted/fake LLM backend and isolated workspace,
when wave-1 E2E scenarios run,
then gateway lifecycle and agent session flow scenarios pass without external network dependency.

### AC-3 Ops dashboard live control/data conformance remains intact
Given ops dashboard shell routes and live runtime artifacts,
when conformance tests execute,
then live control actions and live status/read models remain contract-valid and regression-protected.

### AC-4 Verification outputs provide auditable RED/GREEN/REGRESSION evidence
Given wave-1 implementation PR verification,
when tests/scripts run,
then RED failure evidence, GREEN pass evidence, and regression outputs are captured and linked.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M298 milestone + issue hierarchy | inspect spec artifacts | all required files/sections exist |
| C-02 | AC-2 | Integration | isolated test workspace + scripted LLM | run wave-1 E2E tests | deterministic pass with no live provider calls |
| C-03 | AC-3 | Functional/Integration | ops shell live data + control routes | run dashboard/gateway conformance tests | control/data markers + mutation paths pass |
| C-04 | AC-4 | Regression | PR verification commands | run RED->GREEN->REGRESSION sequence | evidence is recorded and complete |

## Success Metrics / Observable Signals
- Wave-1 E2E harness tests pass in local/CI deterministic mode.
- Dashboard live conformance tests pass with no route/control regressions.
- Verification artifacts include complete AC->test mapping and tier matrix.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `ls -1 specs/milestones/m298/index.md specs/3448/spec.md specs/3448/plan.md specs/3448/tasks.md` lists all required artifact paths. |
| AC-2 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_3448_ -- --nocapture` passed both wave-1 tests, including `integration_spec_3448_c02_tau_e2e_harness_runs_gateway_lifecycle_and_session_flow`. |
| AC-3 | ✅ | `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_3448_ -- --nocapture` passed `integration_spec_3448_c03_tau_e2e_harness_keeps_ops_control_and_dashboard_live_contracts`; regression checks also passed for `integration_ops_shell_control_action_form_submits_dashboard_mutation_and_redirects` and `functional_spec_2810_c01_c02_c03_ops_shell_control_markers_reflect_dashboard_control_snapshot`. |
| AC-4 | ✅ | RED evidence: `cargo test -p tau-gateway integration_spec_3448_c02_tau_e2e_harness_runs_gateway_lifecycle_and_session_flow -- --nocapture` failed with missing `TauE2eHarness`/`scripted_gateway_response`; GREEN evidence: `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway integration_spec_3448_ -- --nocapture` passed with 2/2 tests; regression+format+lint evidence recorded in `specs/3448/tasks.md` including `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-gateway --tests --no-deps -- -D warnings`. |
