# Spec: Issue #3703 - Collapse scaffold separator whitespace when deleting active placeholders with Backspace/Delete in `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now normalizes separator whitespace when an active
scaffold placeholder is deleted with word-deletion shortcuts. The character
deletion paths still lag behind: deleting an active placeholder with `Delete`
or `Backspace` removes only the placeholder token and leaves redundant
separator whitespace behind. That produces awkward command lines like
`copy  <dest>` or `copy <source> ` instead of clean operator-grade command
text.

## Scope
In scope:
- active-placeholder `Delete` spacing cleanup
- active-placeholder `Backspace` spacing cleanup
- preserving non-placeholder character deletion behavior
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3703/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- word deletion semantics
- placeholder movement semantics
- runtime/gateway changes
- new command-palette commands or preview UX

## Acceptance Criteria
### AC-1 Delete from an active placeholder collapses redundant separator whitespace
Given a scaffold placeholder is active in the command palette,
when the operator presses `Delete`,
then Tau removes the active placeholder and normalizes adjacent separator
whitespace so the remaining arguments stay single-spaced with no doubled gap.

### AC-2 Backspace from an active placeholder trims redundant trailing separator whitespace
Given a scaffold placeholder is active in the command palette,
when the operator presses `Backspace`,
then Tau removes the active placeholder and trims any redundant trailing
separator whitespace so the remaining command text stays clean.

### AC-3 Existing placeholder and character-deletion flows do not regress
Given existing command-palette placeholder focus and character-deletion
behavior,
when active-placeholder whitespace cleanup lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Delete` from an active first placeholder in `copy <source> <dest>`
  yields `copy <dest>` instead of a doubled separator. Maps to AC-1. Tier:
  Functional.
- C-02 `Backspace` from an active last placeholder in `copy <source> <dest>`
  yields `copy <source>` instead of leaving trailing separator whitespace. Maps
  to AC-2. Tier: Functional.
- C-03 Existing command-palette placeholder and character-deletion coverage
  still passes. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Deleting active scaffold placeholders with character-delete shortcuts leaves
  clean command text with no doubled or trailing spaces
- Character-delete behavior now matches the quality of word-deletion behavior
- Existing command-palette editing flows remain stable

## Key Decisions
- Limit whitespace normalization to active-placeholder character-deletion
  branches only
- Preserve ordinary non-placeholder character-deletion semantics
- Keep the slice scoped to command-text cleanup rather than scaffold redesign
