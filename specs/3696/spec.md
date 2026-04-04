# Spec: Issue #3696 - Add word-wise movement shortcuts to `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports shell-style line movement and kill
shortcuts, but it still lacks word-wise cursor movement. Operators can jump to
line start or end, yet they cannot move backward or forward by words with
`Alt+B` and `Alt+F` the way they can in the main input editor. A stronger REPL
should let operators reshape palette queries and scaffolded commands at word
granularity without falling back to character-by-character cursor movement.

## Scope
In scope:
- word-wise cursor movement inside the command palette
- parity with the main input editor's `Alt+B` and `Alt+F` behavior
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3696/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- word-wise deletion
- gateway/runtime changes
- prompt input editor changes
- new command-palette commands

## Acceptance Criteria
### AC-1 Command palette supports backward word movement
Given the operator is editing multi-word text in the command palette,
when they press `Alt+B`,
then Tau moves the palette cursor backward to the previous word boundary so the
next typed characters insert before that word.

### AC-2 Command palette supports forward word movement
Given the operator is editing multi-word text in the command palette,
when they press `Alt+F`,
then Tau moves the palette cursor forward to the next word boundary so the next
typed characters insert at that location.

### AC-3 Earlier M335 palette browsing, placeholder, and shell-style line editing do not regress
Given the existing command-palette autocomplete, placeholders, reverse
placeholder cycling, shell-style line movement, and kill shortcuts,
when word-wise movement lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Alt+B` moves the command-palette cursor to the previous word boundary
  so typed text inserts before the previous word. Maps to AC-1. Tier:
  Functional.
- C-02 `Alt+F` moves the command-palette cursor to the next word boundary so
  typed text inserts at the next word boundary. Maps to AC-2. Tier:
  Functional.
- C-03 Existing command-palette autocomplete, placeholder, and shell-style line
  editing coverage still passes. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can reshape palette queries with word-level jumps instead of
  repeated left/right movement
- Command-palette editing continues to converge with the main REPL input model
- Existing palette shortcut behavior remains stable

## Key Decisions
- Reuse the main input editor's word-boundary semantics for command-palette
  movement
- Limit this slice to movement only, not word-wise deletion
- Keep word movement inside the current single-line command-palette model
