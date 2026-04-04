# Spec: Issue #3701 - Add placeholder-aware Alt+B/Alt+F word escape to `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports scaffold placeholders, character-wise
cursor escape, and word-wise `Alt+B`/`Alt+F` movement. Those features still do
not compose correctly when a placeholder is active. Word-wise movement treats
the literal placeholder token as ordinary text, so `Alt+B` can jump into the
previous placeholder and `Alt+F` can skip across inter-placeholder whitespace
into the next placeholder. A stronger REPL should treat the active placeholder
as the current atomic span for the first word-wise escape action.

## Scope
In scope:
- placeholder-aware `Alt+B` behavior for an active scaffold placeholder
- placeholder-aware `Alt+F` behavior for an active scaffold placeholder
- preserving ordinary word-wise movement when no placeholder is active
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3701/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- changing `Tab`/`BackTab` placeholder cycling
- character-wise `Left`/`Right` behavior
- delete semantics
- gateway/runtime changes

## Acceptance Criteria
### AC-1 Alt+B escapes an active placeholder to its start boundary
Given a scaffold placeholder is active in the command palette,
when the operator presses `Alt+B`,
then Tau clears active placeholder focus and keeps the cursor at that
placeholder's start boundary so inserted text lands immediately before the
placeholder token instead of jumping into earlier words or placeholders.

### AC-2 Alt+F escapes an active placeholder to its end boundary
Given a scaffold placeholder is active in the command palette,
when the operator presses `Alt+F`,
then Tau clears active placeholder focus and moves the cursor to that
placeholder's end boundary so inserted text lands immediately after the current
placeholder token instead of skipping ahead to the next placeholder.

### AC-3 Ordinary word-wise movement and earlier placeholder flows do not regress
Given existing command-palette word movement, placeholder autofocus,
placeholder cycling, and inline replacement flows,
when placeholder-aware word escape lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Alt+B` from an active later placeholder keeps the cursor at that
  placeholder's start boundary so typed text inserts immediately before it.
  Maps to AC-1. Tier: Functional.
- C-02 `Alt+F` from an active earlier placeholder keeps the cursor at that
  placeholder's end boundary so typed text inserts immediately after it rather
  than at the next placeholder. Maps to AC-2. Tier: Functional.
- C-03 Existing command-palette word movement, placeholder autofocus,
  placeholder cycling, and inline replacement coverage still passes. Maps to
  AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can use word-wise movement around active scaffold placeholders
  without dropping into adjacent placeholder spans
- Placeholder-focused editing remains coherent across character-wise and
  word-wise cursor movement
- Existing palette editing flows remain stable

## Key Decisions
- Treat an active placeholder as an atomic span for the first `Alt+B` or
  `Alt+F` escape action
- Keep ordinary word-wise movement unchanged when no placeholder is active
- Limit the slice to movement semantics rather than deletion or scaffold rewrite
