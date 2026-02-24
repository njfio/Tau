# Spec: Issue #3557 - Interactive in-flight progress indicator for TUI agent turns

Status: Implemented

## Problem Statement
During interactive `tau-tui agent` usage (including `tau-unified.sh tui`), long
prompt turns can appear idle because no explicit in-flight state is emitted
between prompt submission and final response/error.

## Scope
In scope:
- Emit interactive turn start indicator before prompt execution.
- Emit periodic in-flight heartbeat with elapsed time while turn is running.
- Emit turn-end indicator with terminal status.
- Gate indicator to TTY interactive mode to avoid noisy non-interactive output.
- Add tests covering progress indicator emission contract.

Out of scope:
- Full ratatui live rendering integration inside `tau-coding-agent` REPL.
- Dashboard panel redesign.
- Provider/API retry policy changes.

## Acceptance Criteria
### AC-1 Start indicator appears at turn begin in interactive TTY mode
Given operator submits a non-command prompt in interactive TTY mode,
when turn execution starts,
then an explicit progress-start line is emitted before completion/error.

### AC-2 Periodic heartbeat appears while turn is in-flight
Given prompt execution takes longer than one heartbeat interval,
when turn remains in-flight,
then elapsed-time heartbeat lines are emitted until completion/cancel/timeout.

### AC-3 End indicator includes terminal status
Given prompt turn completes, cancels, times out, or fails,
when turn exits,
then an explicit progress-end line includes terminal status and elapsed time.

### AC-4 Non-TTY/scripted runs remain clean by default
Given interactive mode is executed with non-terminal stdin/stdout (REPL harness),
when prompt runs,
then progress indicator lines are not emitted.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | TTY interactive turn | submit prompt | `interactive.turn=start ...` appears |
| C-02 | AC-2 | Functional | slow response > heartbeat interval | submit prompt | `interactive.turn=running ... elapsed_ms=...` appears |
| C-03 | AC-3 | Functional | successful/failed turn | prompt exits | `interactive.turn=end status=...` appears |
| C-04 | AC-4 | Integration/Regression | non-tty REPL harness | run fixture | no `interactive.turn=` output |

## Success Metrics / Observable Signals
- Operators no longer face “blank waiting cursor” ambiguity during turn
  execution.
- Heartbeat lines provide deterministic activity confirmation for slow turns.
- Existing non-tty harness output remains stable (no unexpected indicator noise).

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | Added start marker via `format_interactive_turn_start_line` and `InteractiveTurnProgressTracker::start` in `crates/tau-coding-agent/src/runtime_loop.rs`. |
| AC-2 | ✅ | Added heartbeat interval output (`interactive.turn=running elapsed_ms=...`) in tracker task loop. |
| AC-3 | ✅ | Added end marker with terminal status (`completed/cancelled/timed-out/failed`) through tracker finish wiring in both interactive paths. |
| AC-4 | ✅ | TTY gating helper `should_emit_interactive_turn_progress` only enables markers for terminal stdin/stdout; REPL harness fixtures assert `stderr_not_contains: interactive.turn=` and pass. |
