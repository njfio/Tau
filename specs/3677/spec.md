# Spec: Issue #3677 - Raise `tau-tui` to a world-class operator REPL

Status: Implemented

## Problem Statement
Tau's interactive TUI already supports chat, tool streaming, mission controls,
and slash commands, but it still behaves more like a thin terminal frontend
than a high-leverage REPL. The highest-value gap for the first slice is runtime
control: operators cannot inspect live gateway state from inside the TUI,
quickly retry the last prompt, or detach from a resumed mission binding without
dropping to logs or restarting the session. Tau needs a first REPL-focused
upgrade that makes runtime state legible and recovery keyboard-first without
destabilizing the existing gateway and Ralph-loop runtime.

## Scope
In scope:
- `crates/tau-tui/src/interactive/*`
- interactive runtime control and operator-visible runtime state
- slash-command workflows for runtime inspection and recovery
- a staged REPL upgrade, with this story implementing the runtime control and
  observability slice first
- spec/plan/tasks updates under `specs/3677/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- prompt editor/history upgrades
- transcript search/copy mode upgrades
- replacing the gateway or Ralph-loop runtime model
- cosmetic redesign without workflow improvement
- web dashboard redesign
- speculative plugin architecture changes

## Acceptance Criteria
### AC-1 Runtime inspection is available inside the TUI
Given an operator is using `tau-tui interactive`,
when they run `/status`,
then Tau fetches the live gateway status snapshot, renders a concise operator
summary in the transcript, and updates the status surface with runtime health
signals including heartbeat state and queue/backlog information when present.

### AC-2 Recovery controls stay inside the REPL
Given an operator has already submitted a prompt or resumed a mission,
when they run `/retry` or `/detach`,
then `/retry` resubmits the latest user prompt through the existing gateway
turn flow and `/detach` clears the active mission binding while keeping the TUI
session alive.

### AC-3 Live status remains legible during long-running turns
Given a gateway-backed interactive turn is in progress or has just failed,
when the operator watches the TUI status/help surfaces,
then they can see the latest runtime snapshot, active mission context, and the
available recovery affordances without needing raw gateway logs.

### AC-4 Existing gateway-backed TUI behavior does not regress
Given existing interactive gateway flows for chat, tool streaming, and mission
controls,
when the runtime-control REPL slice is implemented,
then existing behaviors continue to work and the new affordances are covered by
scoped tests.

## Conformance Cases
- C-01 /status fetches `/gateway/status`, prints a concise runtime summary, and
  surfaces heartbeat state and queue depth. Maps to AC-1. Tier: Functional.
- C-02 /retry resubmits the latest user prompt and preserves the normal gateway
  turn request path. Maps to AC-2. Tier: Functional.
- C-03 /detach clears `mission_id` from the active gateway config and status
  bar, then confirms the detach in the transcript. Maps to AC-2. Tier:
  Functional.
- C-04 Status/help rendering exposes live runtime control hints and the latest
  runtime snapshot without breaking existing mission visibility. Maps to AC-3.
  Tier: Functional.
- C-05 Existing `/missions`, `/mission`, `/resume`, and streamed turn handling
  still pass after the new slice lands. Maps to AC-4. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can inspect runtime state, retry, and detach without leaving the
  TUI
- Long-running turns become easier to understand without reading raw logs
- Existing gateway/TUI tests remain green and new REPL tests cover C-01..C-05

## Scope Boundaries
- This story defines and implements the runtime-control first slice, not the
  final end-state for every possible shell feature
- Editor/history and transcript-power upgrades remain follow-up work under M335

## Key Decisions
- REPL quality should be judged by operator throughput and clarity, not by a
  visual reskin alone
- The first implementation slice targets runtime control and observability
  rather than editor or transcript ergonomics
