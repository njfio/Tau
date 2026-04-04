# Spec: Issue #3700 - Add placeholder-aware Left/Right cursor escape to `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports scaffold placeholders plus
character-wise `Left` and `Right` cursor movement, but those behaviors still
conflict. Once a placeholder is active, `Left` and `Right` behave like raw
character motion, which drops the operator into the literal `<placeholder>`
token text instead of treating the placeholder as an atomic scaffold span. A
stronger REPL should let operators escape an active placeholder cleanly at its
start or end boundary before resuming ordinary character editing.

## Scope
In scope:
- placeholder-aware `Left` behavior at an active scaffold placeholder
- placeholder-aware `Right` behavior at an active scaffold placeholder
- preserving ordinary character-wise left/right movement when no placeholder is
  active
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3700/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- changing `Tab`/`BackTab` placeholder cycling
- gateway/runtime changes
- prompt input editor changes
- placeholder-aware word deletion

## Acceptance Criteria
### AC-1 Left escapes an active placeholder to its start boundary
Given a scaffold placeholder is active in the command palette,
when the operator presses `Left`,
then Tau clears active placeholder focus but keeps the cursor at that
placeholder's start boundary so typed characters insert immediately before the
placeholder token.

### AC-2 Right escapes an active placeholder to its end boundary
Given a scaffold placeholder is active in the command palette,
when the operator presses `Right`,
then Tau clears active placeholder focus and moves the cursor to that
placeholder's end boundary so typed characters insert immediately after the
placeholder token.

### AC-3 Ordinary character-wise cursor movement and earlier placeholder flows do not regress
Given the existing command-palette left/right cursor movement, placeholder
autofocus, placeholder cycling, and inline replacement flows,
when placeholder-aware escape lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Left` from an active second placeholder keeps the cursor at that
  placeholder's start boundary so typed text inserts immediately before it.
  Maps to AC-1. Tier: Functional.
- C-02 `Right` from an active placeholder moves the cursor to the placeholder's
  end boundary so typed text inserts immediately after it. Maps to AC-2. Tier:
  Functional.
- C-03 Existing command-palette cursor movement, placeholder autofocus,
  placeholder cycling, and inline replacement coverage still passes. Maps to
  AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can move around scaffold placeholders without exposing or editing
- raw placeholder token text one character at a time
- Placeholder-focused editing feels coherent with the rest of the palette editor
- Existing placeholder and cursor behaviors remain stable

## Key Decisions
- Treat an active placeholder as an atomic span for the first `Left` or `Right`
  escape action
- Keep ordinary character-wise `Left`/`Right` movement unchanged when no
  placeholder is active
- Limit the slice to cursor escape only, not placeholder-aware delete semantics
