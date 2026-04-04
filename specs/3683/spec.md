# Spec: Issue #3683 - Add command palette autocomplete and history to `tau-tui` REPL

Status: Implemented

## Problem Statement
`tau-tui` has a command palette, but it still behaves like a bare input box. It
does not show available commands, it cannot autocomplete them, and it forgets
previous palette commands. A world-class REPL needs a discoverable,
keyboard-first command surface that makes runtime control fast instead of
requiring memorized slash commands.

## Scope
In scope:
- command palette suggestions with command descriptions
- keyboard navigation across matching command suggestions
- `Tab` autocomplete and enter-to-run selected suggestion
- command-palette-local command history recall
- TDD coverage for palette suggestion, autocomplete, and history behavior
- spec/plan/tasks updates under `specs/3683/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- fuzzy ranking across transcript content
- mouse selection inside the palette
- multi-column command browsing
- changes to gateway protocols

## Acceptance Criteria
### AC-1 The command palette shows matching suggestions
Given the operator opens the command palette,
when they type part of a command,
then Tau renders matching command suggestions with short descriptions.

### AC-2 The operator can autocomplete and run commands from the palette
Given the command palette has matching suggestions,
when the operator uses `Up/Down` and `Tab` or presses `Enter`,
then Tau selects the intended command and executes it from the keyboard.

### AC-3 The command palette remembers previous palette commands
Given the operator has executed commands from the palette,
when they reopen it and recall history,
then Tau restores earlier palette commands for reuse/editing.

### AC-4 Earlier M335 REPL slices do not regress
Given runtime control, prompt history, transcript search/copy, local session
and transcript persistence, and transcript export already exist,
when command palette autocomplete/history lands,
then those behaviors still pass in scoped regression coverage.

## Conformance Cases
- C-01 Opening the palette and typing a partial command renders matching
  suggestions. Maps to AC-1. Tier: Functional.
- C-02 Palette `Up/Down`, `Tab`, and `Enter` select and execute the expected
  command. Maps to AC-2. Tier: Functional.
- C-03 Palette command history recall restores previously executed commands.
  Maps to AC-3. Tier: Functional.
- C-04 Existing `#3677`, `#3678`, `#3679`, `#3680`, `#3681`, and `#3682` TUI
  regression tests continue to pass. Maps to AC-4. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can discover commands from the palette without leaving the TUI
- Command execution from the palette takes fewer keystrokes than manual typing
- Repeated runtime-control commands are recallable from palette history

## Key Decisions
- The palette uses a stable command catalog with short descriptions
- Suggestion navigation is keyboard-only: `Up/Down` selects, `Tab`
  autocompletes, `Enter` executes
- Palette history is independent from prompt history
