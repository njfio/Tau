# Spec: Issue #3682 - Add transcript export commands to `tau-tui` REPL

Status: Implemented

## Problem Statement
`tau-tui` can now search, copy the latest transcript entry, and persist the
session locally, but operators still cannot export the whole current REPL
session cleanly. A world-class REPL needs a fast way to copy or save the full
transcript so work can be shared, archived, or moved into docs without leaving
the terminal.

## Scope
In scope:
- slash commands to copy the current transcript to the clipboard
- slash commands to save the current transcript to disk
- a stable text export format for the current transcript
- operator-facing success/error feedback in the TUI
- TDD coverage for transcript copy/save behavior
- spec/plan/tasks updates under `specs/3682/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- rich HTML or PDF export
- remote transcript sync
- transcript import/replay
- changing gateway protocols

## Acceptance Criteria
### AC-1 Operators can copy the full current transcript
Given the interactive TUI contains transcript messages,
when the operator runs the transcript-copy command,
then Tau copies the formatted transcript to the clipboard and confirms success
in the chat.

### AC-2 Operators can save the full current transcript to disk
Given the interactive TUI contains transcript messages,
when the operator runs the transcript-save command,
then Tau writes the formatted transcript to a file and confirms the saved path
in the chat.

### AC-3 Transcript export errors fail loud but do not crash the TUI
Given clipboard or file-write export fails,
when the operator runs a transcript export command,
then Tau surfaces a clear system error message and the TUI keeps running.

### AC-4 Earlier M335 REPL slices do not regress
Given runtime control, prompt history, search/copy, local session persistence,
and local transcript persistence already exist,
when transcript export commands land,
then those behaviors still pass in scoped regression coverage.

## Conformance Cases
- C-01 `/copy-transcript` copies the formatted transcript for the active
  session. Maps to AC-1. Tier: Functional.
- C-02 `/save-transcript` writes a transcript file and confirms the saved path.
  Maps to AC-2. Tier: Functional.
- C-03 Clipboard or write failures surface a system error message without
  crashing. Maps to AC-3. Tier: Functional.
- C-04 Existing `#3677`, `#3678`, `#3679`, `#3680`, and `#3681` TUI regression
  tests continue to pass after transcript export lands. Maps to AC-4. Tier:
  Regression.

## Success Metrics / Observable Signals
- Operators can export the current transcript from inside the TUI
- Exported transcript format is readable and stable
- Failed clipboard or file writes do not terminate the TUI

## Key Decisions
- Export format is plain text with timestamps and role labels
- `copy` and `save` reuse a shared transcript renderer so outputs stay
  consistent
- Default save path lives under `.tau/tui/exports/`
