# Spec: Issue #3702 - Collapse scaffold separator whitespace when deleting active placeholders in `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports deleting an active scaffold
placeholder with word-deletion shortcuts like `Alt+D` and `Ctrl+W`. The current
behavior removes only the placeholder token itself and leaves the surrounding
separator whitespace untouched. That produces awkward command lines like
`copy  <dest>` or `copy <source> `, which is not operator-grade REPL behavior.
Deleting an active scaffold placeholder should leave behind a clean single-space
separator layout instead of doubled or trailing spaces.

## Scope
In scope:
- active-placeholder `Alt+D` deletion spacing cleanup
- active-placeholder `Ctrl+W` deletion spacing cleanup
- preserving non-placeholder word deletion behavior
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3702/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- non-placeholder word deletion behavior changes
- placeholder cursor movement changes
- gateway/runtime changes
- new command-palette commands or preview UX

## Acceptance Criteria
### AC-1 Alt+D from an active placeholder collapses redundant separator whitespace
Given a scaffold placeholder is active in the command palette,
when the operator presses `Alt+D`,
then Tau removes the active placeholder and normalizes adjacent separator
whitespace so the remaining arguments stay single-spaced with no doubled gap.

### AC-2 Ctrl+W from an active placeholder trims redundant trailing separator whitespace
Given a scaffold placeholder is active in the command palette,
when the operator presses `Ctrl+W`,
then Tau removes the active placeholder and trims any redundant trailing
separator whitespace so the remaining command text stays clean.

### AC-3 Existing placeholder and word-deletion flows do not regress
Given existing command-palette placeholder focus and word-deletion behavior,
when active-placeholder whitespace cleanup lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Alt+D` from an active first placeholder in `copy <source> <dest>`
  yields `copy <dest>` instead of a doubled separator. Maps to AC-1. Tier:
  Functional.
- C-02 `Ctrl+W` from an active last placeholder in `copy <source> <dest>`
  yields `copy <source>` instead of leaving trailing separator whitespace. Maps
  to AC-2. Tier: Functional.
- C-03 Existing command-palette placeholder, word-movement, and word-deletion
  coverage still passes. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Deleting active scaffold placeholders leaves behind clean command text with no
  doubled or trailing spaces
- Placeholder-focused deletion feels coherent with the rest of the palette
  editor
- Existing command-palette editing flows remain stable

## Key Decisions
- Limit whitespace normalization to active-placeholder deletion branches only
- Preserve ordinary non-placeholder word-deletion semantics
- Keep the slice scoped to command-text cleanup rather than scaffold redesign
