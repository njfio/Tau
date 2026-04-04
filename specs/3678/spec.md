# Spec: Issue #3678 - Add prompt history and editor ergonomics to `tau-tui` REPL

Status: Implemented

## Problem Statement
The first M335 slice added runtime control, but the `tau-tui` input editor is
still too weak for serious iterative work. Operators cannot recall prior
prompts, preserve a half-written draft while browsing history, or use common
shell-style editing controls to move and reshape a prompt quickly. Tau needs a
second REPL slice that makes the input surface feel stateful and efficient
without changing the gateway/runtime contracts.

## Scope
In scope:
- prompt history recall and resend inside `crates/tau-tui/src/interactive/*`
- draft preservation while navigating prompt history
- stronger keyboard editing ergonomics for the existing multiline editor
- TDD coverage for history recall and editor controls
- spec/plan/tasks updates under `specs/3678/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- transcript search or copy mode
- gateway/backend changes
- visual-only redesign
- fuzzy search or reverse-history search

## Acceptance Criteria
### AC-1 Prompt history is available inside the editor
Given an operator has already submitted prior prompts in the current TUI
session,
when they use the history recall keys in insert mode,
then Tau loads older and newer prompts into the editor without sending them and
lets the operator edit or resend the recalled text.

### AC-2 Browsing history preserves the in-progress draft
Given an operator has typed a new unsent draft,
when they enter prompt history and then return to the newest slot,
then the original unsent draft is restored instead of being lost.

### AC-3 The input editor supports stronger shell-style editing controls
Given the operator is editing a prompt in insert mode,
when they use the configured movement and deletion shortcuts,
then Tau supports fast cursor motion and prompt reshaping without forcing
character-by-character editing.

### AC-4 Existing submit and multiline behaviors do not regress
Given existing TUI flows for send, newline insertion, and gateway-backed turns,
when the new editor/history slice is implemented,
then existing behavior still works and the new editor features are covered by
scoped tests.

## Conformance Cases
- C-01 Recalling prompt history loads older prompts, then newer prompts, inside
  the editor without auto-submitting. Maps to AC-1. Tier: Functional.
- C-02 Returning to the newest history slot restores the unsent draft exactly.
  Maps to AC-2. Tier: Functional.
- C-03 Shell-style editing shortcuts move by word/start/end and support fast
  deletion/clearing inside the editor. Maps to AC-3. Tier: Unit/Functional.
- C-04 Existing submit/newline behavior and gateway-backed prompt submission
  continue to pass after the history/editor changes. Maps to AC-4. Tier:
  Regression.

## Success Metrics / Observable Signals
- Operators can recall and resend recent prompts without leaving the TUI
- Unsaved drafts survive temporary history browsing
- Editing common prompt changes takes fewer keystrokes than raw arrow+delete
- Existing TUI gateway tests remain green

## Key Decisions
- This slice should stay local to `tau-tui`; it must not require any gateway
  protocol changes
- History navigation must preserve an in-progress draft rather than overwriting
  it
- Keyboard choices should avoid collisions with the new runtime-control layer;
  use insert-mode editor shortcuts instead of repurposing global commands
