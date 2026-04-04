# Spec: Issue #3706 - Make active scaffold placeholders atomic for `Ctrl+A`/`Ctrl+E` in `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette already treats active scaffold placeholders as
atomic spans for character-wise movement, word-wise movement, deletion, line
clearing, and `Esc` focus clearing. `Ctrl+A` and `Ctrl+E` still bypass that
placeholder-aware model and jump straight to absolute line boundaries. When an
operator is editing an active placeholder, that makes line-boundary movement
inconsistent with the rest of the placeholder-aware editor and forces extra
cursor repair before inserting prefix or suffix text around the scaffold field.

## Scope
In scope:
- active-placeholder `Ctrl+A` start-boundary escape in the command palette
- active-placeholder `Ctrl+E` end-boundary escape in the command palette
- preserving ordinary non-placeholder line-boundary movement
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3706/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- non-command-palette editing semantics
- runtime/gateway changes
- placeholder deletion or line-clearing behavior
- new command-palette commands or overlays

## Acceptance Criteria
### AC-1 `Ctrl+A` from an active placeholder escapes to the placeholder start boundary
Given a scaffold placeholder is active in the command palette,
when the operator presses `Ctrl+A`,
then Tau clears active placeholder focus and places the cursor at that
placeholder's start boundary instead of jumping to the absolute start of the
line.

### AC-2 `Ctrl+E` from an active placeholder escapes to the placeholder end boundary
Given a scaffold placeholder is active in the command palette,
when the operator presses `Ctrl+E`,
then Tau clears active placeholder focus and places the cursor at that
placeholder's end boundary instead of jumping to the absolute end of the line.

### AC-3 Existing command-palette line-boundary behavior does not regress
Given the earlier command-palette editor behavior,
when placeholder-aware `Ctrl+A` and `Ctrl+E` handling lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Ctrl+A` from an active `<dest>` placeholder in `copy <source> <dest>`
  escapes to that placeholder's start boundary so typing inserts `tmp/` before
  `<dest>`. Maps to AC-1. Tier: Functional.
- C-02 `Ctrl+E` from an active `<source>` placeholder in `copy <source> <dest>`
  escapes to that placeholder's end boundary so typing appends `-current` after
  `<source>`. Maps to AC-2. Tier: Functional.
- C-03 Existing command-palette editor coverage still passes after the new
  line-boundary routing lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can use `Ctrl+A` and `Ctrl+E` around active scaffold placeholders
  without being thrown to absolute line boundaries
- Placeholder-aware line-boundary movement feels consistent with existing
  placeholder-aware `Left`/`Right` and `Alt+B`/`Alt+F` behavior
- Existing command-palette editing flows remain stable

## Key Decisions
- Treat active placeholder focus as an atomic editing mode for line-boundary
  movement too
- Keep ordinary non-placeholder `Ctrl+A` and `Ctrl+E` behavior unchanged
- Limit the slice to `Ctrl+A`/`Ctrl+E` rather than broader command-line UX
