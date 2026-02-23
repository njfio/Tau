# Spec: Issue #3428 - Dashboard/TUI operator-flow convergence

Status: Implemented

## Problem Statement
Dashboard routes expose richer operator control signals than the TUI shell currently renders. The TUI omits core control-plane fields (`auth mode/required`, `health reason`, `queue depth`, `failure streak`, and primary alert metadata), making cross-surface workflows inconsistent and harder to validate deterministically.

## Scope
In scope:
- Extend TUI operator-shell view model/rendering to include control-plane parity fields used by dashboard operators.
- Add cross-surface conformance tests that assert dashboard and TUI surface the same core operator-flow signals from the same synthetic snapshot.
- Keep changes deterministic and local to dashboard/TUI presentation contracts.

Out of scope:
- New dashboard API routes or gateway wire-contract changes.
- Live network fetching in `tau-tui`.
- Visual redesign of dashboard web UI.

## Acceptance Criteria
### AC-1 TUI renders explicit auth-mode parity signals
Given operator shell data includes auth mode and auth requirement,
when `render_operator_shell_frame` is called,
then rendered output includes deterministic auth mode and auth-required markers.

### AC-2 TUI renders control-plane health and queue parity signals
Given operator shell data includes health reason, queue depth, failure streak, and primary alert metadata,
when `render_operator_shell_frame` is called,
then rendered output includes those fields in deterministic panel text.

### AC-3 Cross-surface parity tests lock shared operator-flow semantics
Given dashboard and TUI are fed the same synthetic operator snapshot,
when both surfaces are rendered,
then both outputs contain matching values for auth mode/required, health state/reason, queue depth, failure streak, and primary alert code.

### AC-4 Spec/process artifacts and verification evidence are complete
Given AGENTS contract requirements,
when issue artifacts and tests are reviewed,
then `spec.md`, `plan.md`, and `tasks.md` exist with AC/conformance mappings and RED/GREEN/REGRESSION evidence.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Unit/Conformance | shell frame has auth mode + required flag | render shell frame | output includes `auth.mode` and `auth.required` markers |
| C-02 | AC-2 | Unit/Conformance | shell frame has health reason, queue depth, failure streak, primary alert fields | render shell frame | output includes deterministic control-plane markers |
| C-03 | AC-3 | Integration/Conformance | shared synthetic snapshot values across dashboard + TUI | render dashboard HTML and TUI frame | both outputs include same key operator-flow values |
| C-04 | AC-4 | Conformance | issue artifact set | verify paths/sections | spec/plan/tasks exist with evidence mapping |

## Success Metrics / Observable Signals
- TUI shell output includes parity fields without breaking existing panel contract.
- Cross-surface parity test passes and fails on mismatched operator-flow values.
- Targeted dashboard/TUI tests and quality gates are green.
