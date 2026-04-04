# Spec: Issue #3695 - Add shell-style cursor and kill shortcuts to `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports autocomplete, scaffolds,
placeholder-aware editing, reverse placeholder cycling, and active placeholder
feedback, but editing inside the palette is still primitive compared with the
main input editor. Operators can type and backspace, but they cannot use
shell-style line editing shortcuts like `Ctrl+A`, `Ctrl+E`, `Ctrl+U`, and
`Ctrl+K` while refining queries or scaffolded commands. A stronger REPL should
offer the same core line-editing ergonomics inside the command palette.

## Scope
In scope:
- shell-style line-start and line-end cursor movement in the command palette
- shell-style clear-to-start and clear-to-end editing in the command palette
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3695/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- new palette commands
- gateway/runtime changes
- prompt input editor changes
- word-wise movement and deletion

## Acceptance Criteria
### AC-1 Command palette supports shell-style line-start and line-end movement
Given the operator is editing text in the command palette,
when they press `Ctrl+A` or `Ctrl+E`,
then Tau moves the palette cursor to the start or end of the current line so
subsequent typed characters insert at that location.

### AC-2 Command palette supports shell-style clear-to-start and clear-to-end
Given the operator is editing text in the command palette,
when they press `Ctrl+U` or `Ctrl+K`,
then Tau clears text from the cursor to the start or end of the current line
without closing the palette or regressing autocomplete behavior.

### AC-3 Earlier M335 palette browsing, scaffolding, and placeholder behavior do not regress
Given the existing command-palette autocomplete, history, grouping, paging,
placeholder editing, reverse placeholder cycling, and active placeholder
feedback,
when shell-style cursor and kill shortcuts land,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Ctrl+A` and `Ctrl+E` move the command-palette cursor so typed
  characters insert at the start or end of the current line. Maps to AC-1.
  Tier: Functional.
- C-02 `Ctrl+U` clears from the cursor to the line start and `Ctrl+K` clears
  from the cursor to the line end inside the command palette. Maps to AC-2.
  Tier: Functional.
- C-03 Existing command-palette autocomplete, placeholder, and broader
  `interactive::app_gateway_tests` coverage still pass. Maps to AC-3. Tier:
  Regression.

## Success Metrics / Observable Signals
- Operators can reshape palette queries and scaffolds without falling back to
  repeated backspace or full retyping
- Command-palette editing feels consistent with the main input REPL controls
- Existing palette navigation and placeholder workflows remain stable

## Key Decisions
- Reuse the same shell-style shortcut conventions already exposed in the main
  input editor
- Keep the command palette single-line and apply clear operations against the
  single command line only
- Limit this slice to line-wise editing rather than adding word-wise movement
  and deletion at the same time
