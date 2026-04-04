# Spec: Issue #3679 - Add transcript search, copy, and stronger scrollback to `tau-tui` REPL

Status: Implemented

## Problem Statement
`tau-tui` now supports runtime control and prompt history, but transcript power
is still weak. Operators cannot search prior transcript content, quickly copy a
useful transcript entry to the clipboard, or use familiar page-style scrollback
controls in the chat panel. Tau needs a third REPL slice that makes transcript
inspection and reuse faster without changing any gateway/backend contract.

## Scope
In scope:
- transcript search and match navigation inside `crates/tau-tui/src/interactive`
- transcript copy helpers for recent chat entries
- stronger page-style scrollback controls for the chat panel
- TDD coverage for search, copy, and scrollback
- spec/plan/tasks updates under `specs/3679/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- backend/gateway changes
- persistent search history
- full mouse text selection
- fuzzy search or regex search

## Acceptance Criteria
### AC-1 Transcript search works inside the TUI
Given the transcript already contains multiple messages,
when the operator runs transcript search commands,
then Tau finds matching messages, focuses one match at a time, and surfaces the
active search query plus match position in the TUI.

### AC-2 Operators can copy recent transcript output without leaving the TUI
Given the transcript contains useful chat content,
when the operator runs the transcript copy command,
then Tau copies the latest matching transcript entry to the clipboard and
confirms the action in the shell.

### AC-3 Chat scrollback supports stronger keyboard navigation
Given the chat panel has enough messages to scroll,
when the operator uses page-style scroll controls in normal mode,
then Tau moves through transcript history faster than single-line `j/k`
navigation.

### AC-4 Existing runtime-control and prompt-history behaviors do not regress
Given the earlier M335 REPL slices are already implemented,
when transcript power is added,
then existing TUI command, runtime, and editor/history behavior still passes.

## Conformance Cases
- C-01 `/search <query>` sets transcript search state, focuses a match, and
  renders a `Search:` summary with query and active match position. Maps to
  AC-1. Tier: Functional.
- C-02 `/search-prev`, `/search-next`, and `/search-clear` navigate or clear
  the current search state without leaving the TUI. Maps to AC-1. Tier:
  Functional.
- C-03 `/copy-last` with an optional role filter copies the latest matching chat
  message content through the clipboard command and confirms success. Maps to
  AC-2. Tier: Functional.
- C-04 `PageUp` and `PageDown` move the chat scroll offset by a page-sized step
  in normal mode. Maps to AC-3. Tier: Functional.
- C-05 Existing gateway-backed TUI tests and the prior `#3677/#3678` REPL
  slices still pass after the transcript changes. Maps to AC-4. Tier:
  Regression.

## Success Metrics / Observable Signals
- Operators can find and revisit prior transcript content without raw logs
- Recent assistant/system/tool output can be copied from inside the TUI
- Chat navigation feels faster on longer sessions
- Existing M335 TUI regression coverage remains green

## Key Decisions
- Transcript search is local to the TUI process and operates on chat messages
  already present in memory
- Search should expose an explicit in-shell summary rather than relying only on
  styling
- Copy helpers should reuse the existing clipboard command mechanism instead of
  introducing a second clipboard path
