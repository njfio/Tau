# Spec: Issue #3699 - Add Left/Right cursor movement and Delete key to `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports shell-style line movement,
word-wise movement, and word deletion, but it still lacks plain `Left`,
`Right`, and forward `Delete` editing. Operators can jump to the start or end
of the line and kill words, yet they cannot make precise character-level edits
at the current cursor position like they would in a normal REPL line editor.

## Scope
In scope:
- left and right cursor movement inside the command palette
- forward delete at the current command-palette cursor position
- parity with the palette's existing single-line editing model
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3699/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- prompt input editor changes
- gateway/runtime changes
- new command-palette commands
- multi-line palette editing

## Acceptance Criteria
### AC-1 Command palette supports character-wise left and right cursor movement
Given the operator is editing command-palette text,
when they press `Left` or `Right`,
then Tau moves the palette cursor by one character boundary so the next typed
characters insert at that exact position.

### AC-2 Command palette supports forward delete at the current cursor
Given the operator positions the command-palette cursor before a character,
when they press `Delete`,
then Tau removes the character at the cursor and keeps the cursor anchored at
that boundary so the next typed characters replace it.

### AC-3 Earlier M335 palette placeholder, line editing, and word-editing behavior do not regress
Given the existing command-palette autocomplete, placeholders, shell-style line
editing, word movement, and word deletion flows,
when `Left`, `Right`, and `Delete` land,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Left` and `Right` move the command-palette cursor by one character so
  typed text inserts at the expected location. Maps to AC-1. Tier: Functional.
- C-02 `Delete` at an interior command-palette cursor removes the character at
  that cursor and allows replacement text to insert at the same boundary. Maps
  to AC-2. Tier: Functional.
- C-03 Existing command-palette autocomplete, placeholder, shell-style line
  editing, word movement, and word deletion coverage still passes. Maps to
  AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can make precise character-level palette edits without falling back
  to destructive clears or repeated backspaces
- Command-palette editing behaves more like a normal REPL line editor
- Existing palette editing and placeholder behavior remains stable

## Key Decisions
- Keep the slice limited to character-wise cursor motion and forward delete
- Reuse the current single-line command-palette cursor model
- Preserve existing placeholder-clearing semantics unless the cursor is
  operating on ordinary text
