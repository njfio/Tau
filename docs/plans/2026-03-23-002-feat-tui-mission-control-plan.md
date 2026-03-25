---
title: feat: Build TUI Mission Control
type: feat
status: active
date: 2026-03-23
---

# feat: Build TUI Mission Control

## Overview

Upgrade `tau-tui` from a chat-first terminal client into a true mission-control interface for autonomous work. The TUI should expose plan state, approvals, memory, tool traces, background execution, recovery prompts, and operator rationale so a terminal-first operator can supervise the full lifecycle of a mission.

## Problem Statement / Motivation

The current TUI already has strong building blocks:

- full-screen interactive mode
- operator shell and `shell-live` modes
- chat, tools, status bar, command palette, keyboard and mouse handling

But the interactive path currently acts as a thin gateway client that mostly submits a request and renders `output_text`. The operator cannot naturally inspect why the agent chose a path, where it is in a plan, what approvals are pending, which memory/context mattered, or what recovery action is proposed next.

## Proposed Solution

Reframe the TUI around mission supervision with a deliberate multi-pane workflow:

1. Add a mission header showing mission state, budget, checkpoint status, agent state, and environment.
2. Replace the current fixed chat/tools emphasis with a layout that can show:
   - mission summary
   - step graph / current plan lane
   - transcript and tool trace
   - approvals / pending actions
   - memory and rationale
   - alerts / recovery
3. Add explicit operator commands for approve, deny, retry, replan, pause, resume, and inspect.
4. Surface why the system is blocked or what it needs from the operator.
5. Keep shell and shell-live modes useful as lightweight views while making full-screen interactive mode the primary cockpit.

## Implementation Phases

### Phase 1: TUI Information Architecture

- Define the core mission-control layout and focus model.
- Decide which panes are always visible versus overlay-based.
- Specify keyboard and mouse interactions for approvals, history, and inspection.

### Phase 2: Mission Data Consumption

- Extend TUI state to consume mission-oriented runtime state rather than only `output_text` and token usage.
- Render plan progress, current step, pending approvals, and alert/recovery signals.
- Introduce richer build/progress evidence than the existing read-only vs mutating summary.

### Phase 3: Control Actions

- Add mission commands to the command palette and keybindings.
- Support approve/deny/retry/replan/pause/resume actions with explicit confirmations where needed.
- Add contextual action hints based on current mission state.

### Phase 4: Polish and Validation

- Optimize layout behavior for narrow and wide terminals.
- Add high-signal visual language for alerts, degraded mode, and recovery.
- Extend deterministic TUI tests to cover navigation, action visibility, and blocked-state rendering.

## Technical Considerations

- The current layout is still a fixed chat/input/tools split; mission control needs a state-driven layout rather than a permanently subordinate side panel.
- Existing shell/live-shell code already renders operator summaries from persisted artifacts; this should be folded into a shared view-model instead of duplicated.
- The TUI must remain usable over SSH and in low-color environments; clarity matters more than decorative density.
- Command palette and keyboard navigation should stay fast and composable; avoid mouse-only control dependencies.

## System-Wide Impact

### Interaction Graph

Mission state from the runtime backbone feeds TUI view models, which render plan and alert state, which expose operator actions, which call gateway/control paths, which update persisted mission state and event streams.

### Error & Failure Propagation

The TUI must show when the runtime is thinking, blocked, degraded, waiting for approval, or escalating. Hidden failure modes are unacceptable once this becomes a mission-control surface.

### State Lifecycle Risks

UI actions that mutate mission state must be idempotent and reflect the post-action state quickly. TUI lag or stale state could cause duplicate approvals or confusing operator decisions.

### API Surface Parity

The TUI should not invent privileged actions unavailable elsewhere. Control operations should align with gateway/dashboard mission APIs.

### Integration Test Scenarios

- Run an interactive mission and verify the TUI shows plan progress, tool activity, and approval prompts in one session.
- Simulate stale or malformed backing artifacts and verify the TUI shows actionable diagnostics rather than blank panes.
- Trigger recovery/escalation and verify operator actions change state correctly and visibly.

## SpecFlow Notes

### Primary Operator Flows

1. Open a mission and immediately understand current status.
2. Inspect the current plan step and why the system chose it.
3. Approve or deny a pending action without leaving the TUI.
4. Investigate a failure using tool trace, memory, and alert panes.
5. Resume a paused or restarted mission from live persisted state.

### Important Gaps to Resolve in Implementation

- Which panes should be primary on small terminals.
- Whether approvals live in the command palette, dedicated pane, or both.
- How much transcript detail should remain visible by default once mission panes exist.

### Default Planning Assumptions

- Full-screen interactive mode becomes the canonical mission-control experience.
- `shell` and `shell-live` remain lighter-weight summary views.
- Mission actions are explicit, reversible when safe, and audit-visible.

## Acceptance Criteria

- [ ] `tau-tui` can render mission summary, current plan progress, alerts, and pending actions in interactive mode.
- [ ] The TUI exposes operator commands for approve, deny, retry, replan, pause, and resume where supported by the runtime.
- [ ] The TUI shows why the agent is blocked or escalating rather than only showing raw assistant text.
- [ ] `shell-live` and full-screen views share a common mission/state model where practical.
- [ ] Keyboard-first workflows remain complete; mouse support enhances but does not gate core operations.
- [ ] Deterministic tests cover layout, action visibility, degraded states, and artifact diagnostics.
- [ ] Docs describe the mission-control interaction model and terminal expectations.

## Success Metrics

- Operators can complete the common supervision loop from the TUI without switching to the dashboard or raw artifact files.
- Failure and approval states become discoverable in one or two focus changes rather than requiring multiple commands or logs.
- Live mission state is understandable within a few seconds of opening the interface.

## Dependencies & Risks

### Dependencies

- [ ] Governed Mission Mode for the core work object.
- [ ] Unified Runtime State Backbone for reliable data supply.
- [ ] Gateway control/action endpoints for mission-safe mutations.

### Risks

- UI ambition can outrun the quality of backing state and create a flashy but unreliable cockpit.
- Pane overload can make the TUI harder to use if prioritization is weak.
- Terminal constraints will punish overly dense layouts.

## Sources & References

- Source ideation: `docs/ideation/2026-03-23-autonomous-operator-mission-control-ideation.md`
- TUI modes and CLI surface: `crates/tau-tui/src/main.rs:24`
- Interactive app state: `crates/tau-tui/src/interactive/app.rs:17`
- Interactive runtime loop: `crates/tau-tui/src/interactive/app_runtime.rs:16`
- Current layout: `crates/tau-tui/src/interactive/ui_layout.rs`
- Current status bar: `crates/tau-tui/src/interactive/ui_status.rs`
- Current tool panel: `crates/tau-tui/src/interactive/ui_tools.rs`
- Operator shell frame: `crates/tau-tui/src/lib.rs:572`
- Live shell rendering: `crates/tau-tui/src/lib.rs:1016`
- TUI improvement baseline: `docs/planning/integration-gap-closure-plan.md`
