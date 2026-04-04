# Spec: Issue #3697 - Add word-wise deletion shortcut to `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports shell-style line movement, kill
shortcuts, and word-wise cursor movement, but it still cannot delete the
previous word with `Ctrl+W`. Operators can jump by words, yet refining a
palette query or scaffolded command still requires repeated backspaces or
full-line kills. A stronger REPL should let operators remove the previous word
at the cursor boundary with the same shell-style editing model they already use
elsewhere.

## Scope
In scope:
- backward word deletion inside the command palette via `Ctrl+W`
- parity with the palette's existing shell-style editing semantics
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3697/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- forward word deletion
- prompt input editor changes
- gateway/runtime changes
- new command-palette commands

## Acceptance Criteria
### AC-1 Command palette supports deleting the previous word at line end
Given the operator is editing multi-word text at the end of the command
palette,
when they press `Ctrl+W`,
then Tau deletes the previous word and leaves the cursor at the deletion
boundary so typed characters replace that word.

### AC-2 Command palette supports deleting the previous word before the current cursor
Given the operator has moved the command-palette cursor into the middle of
multi-word text,
when they press `Ctrl+W`,
then Tau deletes the previous word segment relative to that cursor position
without removing the following word.

### AC-3 Earlier M335 palette placeholder, line editing, and word movement behavior do not regress
Given the existing command-palette autocomplete, placeholders, shell-style line
editing, and word movement flows,
when `Ctrl+W` deletion lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Ctrl+W` at the end of `search apples` deletes `apples` and allows
  replacement text to insert at the same boundary. Maps to AC-1. Tier:
  Functional.
- C-02 `Ctrl+W` after moving the cursor before `apples` in `search apples`
  deletes `search ` while preserving `apples`, and replacement text inserts at
  the new boundary. Maps to AC-2. Tier: Functional.
- C-03 Existing command-palette autocomplete, placeholder, shell-style line
  editing, and word movement coverage still passes. Maps to AC-3. Tier:
  Regression.

## Success Metrics / Observable Signals
- Operators can remove the prior word in palette queries without repeated
  backspaces
- Command-palette editing continues to converge with a shell-quality REPL model
- Existing palette editing and placeholder behavior remains stable

## Key Decisions
- Reuse shell-style backward word-boundary semantics instead of inventing a new
  deletion model
- Keep the slice limited to backward word deletion only
- Preserve the current single-line command-palette editing model
