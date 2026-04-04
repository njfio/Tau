# Spec: Issue #3698 - Add forward word deletion shortcut to `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports shell-style line movement,
word-wise movement, and backward word deletion, but it still cannot delete the
next word with `Alt+D`. Operators can move to the next word or delete the
previous one, yet refining a palette query or scaffolded command still lacks
the forward kill-word action that completes a shell-quality word-editing set.

## Scope
In scope:
- forward word deletion inside the command palette via `Alt+D`
- parity with the palette's existing shell-style word-editing semantics
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3698/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- prompt input editor changes
- gateway/runtime changes
- new command-palette commands
- non-word deletion behaviors

## Acceptance Criteria
### AC-1 Command palette supports deleting the next word at line start
Given the operator is at the start of multi-word command-palette text,
when they press `Alt+D`,
then Tau deletes the next word segment and leaves the cursor at the same
boundary so typed characters replace that word.

### AC-2 Command palette supports deleting the next word before following text
Given the operator has moved the command-palette cursor to the start of a word
in the middle of multi-word text,
when they press `Alt+D`,
then Tau deletes that word segment and preserves the following word so typed
characters replace only the deleted word.

### AC-3 Earlier M335 palette placeholder, line editing, word movement, and backward word deletion behavior do not regress
Given the existing command-palette autocomplete, placeholders, shell-style line
editing, word movement, and backward word deletion flows,
when `Alt+D` deletion lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Alt+D` at the start of `search apples` deletes `search ` and allows
  replacement text to insert at the same boundary. Maps to AC-1. Tier:
  Functional.
- C-02 `Alt+D` at the start of `apples` in `search apples today` deletes
  `apples ` while preserving `today`, and replacement text inserts at the same
  boundary. Maps to AC-2. Tier: Functional.
- C-03 Existing command-palette autocomplete, placeholder, shell-style line
  editing, word movement, and backward word deletion coverage still passes.
  Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can delete the next word in palette queries without falling back to
  repeated deletes or full-line clears
- Command-palette word editing behaves like a more complete shell-quality REPL
- Existing palette editing and placeholder behavior remains stable

## Key Decisions
- Reuse shell-style forward word-boundary semantics instead of inventing a new
  deletion model
- Keep the slice limited to forward word deletion only
- Preserve the current single-line command-palette editing model
