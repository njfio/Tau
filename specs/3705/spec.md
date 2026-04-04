# Spec: Issue #3705 - Make `Esc` clear active placeholder focus before closing `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports placeholder-aware editing, movement,
and deletion, but `Esc` still behaves like a blunt close action. When a scaffold
placeholder is active, pressing `Esc` immediately closes the palette instead of
first clearing placeholder focus and leaving the operator inside the command
palette. That makes placeholder editing feel inconsistent with the rest of the
REPL editor and forces operators to reopen the palette when they only meant to
leave placeholder mode.

## Scope
In scope:
- active-placeholder `Esc` focus clearing inside the command palette
- follow-up `Esc` palette closing when no placeholder is active
- preserving ordinary command-palette close behavior
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3705/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- insert-mode input `Esc` semantics outside the command palette
- runtime/gateway changes
- new command-palette commands or overlays
- placeholder movement and deletion semantics beyond `Esc`

## Acceptance Criteria
### AC-1 `Esc` clears active placeholder focus before closing the palette
Given a scaffold placeholder is active in the command palette,
when the operator presses `Esc`,
then Tau clears the active placeholder focus, keeps the command palette open,
and leaves the scaffold text intact.

### AC-2 `Esc` closes the command palette when no placeholder is active
Given the command palette is open without an active scaffold placeholder,
when the operator presses `Esc`,
then Tau closes the command palette using its existing close behavior.

### AC-3 Existing command-palette flows do not regress
Given the earlier command-palette editor and placeholder behavior,
when placeholder-aware `Esc` handling lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Esc` from an active placeholder in `resume <mission-id>` clears active
  placeholder feedback and keeps the command palette open. Maps to AC-1. Tier:
  Functional.
- C-02 A second `Esc` after active placeholder focus is cleared closes the
  command palette. Maps to AC-2. Tier: Functional.
- C-03 Existing command-palette editor coverage still passes after the new
  `Esc` routing lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can back out of placeholder mode without losing the current command
  palette session
- `Esc` behavior feels consistent with the rest of the placeholder-aware editor
- Existing command-palette editing flows remain stable

## Key Decisions
- Treat active placeholder focus as a separate sub-mode inside the open command
  palette
- Keep ordinary command-palette close behavior unchanged once no placeholder is
  active
- Limit the slice to `Esc` handling rather than broader modal editing changes
