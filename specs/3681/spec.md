# Spec: Issue #3681 - Add local transcript persistence to `tau-tui` REPL

Status: Not Integrated (2026-04-23 reclassification — see below)

> **2026-04-23 status correction**: although this spec was previously marked
> "Implemented", an audit discovered that the implementation modules
> (`transcript_state.rs` + `transcript_state_tests.rs`) were introduced to the
> tree in commit `8926bd4a` but were **never referenced from any `mod`
> declaration in any git revision**. The compiler never saw them; the feature
> shipped zero runtime behavior. The orphan files were removed in the
> audit-follow-up cleanup. Re-implementing this feature is pending a dedicated
> spec cycle; see
> `docs/solutions/patterns/fallibility-audit-workspace-2026-04.md`
> (Category A) for the forensic trail.

## Problem Statement
`tau-tui` now restores draft input, prompt history, and active mission/session
binding, but the transcript itself is still disposable. Restarting the REPL
drops prior user, assistant, system, and tool messages, which breaks search,
copy, and scrollback continuity. Tau needs a local transcript persistence slice
so the REPL session survives restart as a durable operator workspace.

## Scope
In scope:
- local transcript persistence for the interactive TUI under `.tau/tui/`
- restore recent transcript messages on startup
- save transcript changes during interaction and on shutdown
- avoid duplicate welcome banners when a transcript is restored
- TDD coverage for transcript save/restore and invalid-state fallback
- spec/plan/tasks updates under `specs/3681/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- gateway-side transcript persistence
- transcript sync across machines or operators
- restoring ephemeral render state such as focus, search cursor, or tool panel
  width
- exporting transcripts to external formats

## Acceptance Criteria
### AC-1 Transcript messages are restored on startup
Given a previous `tau-tui interactive` session wrote local transcript state,
when the TUI starts again,
then user, assistant, system, and tool messages are restored into the chat
panel and remain searchable/copyable.

### AC-2 Transcript changes are saved locally without changing gateway behavior
Given the operator sends prompts or Tau appends streamed/runtime messages,
when the transcript changes,
then Tau writes an updated local transcript snapshot and existing gateway
request behavior remains unchanged.

### AC-3 Missing or invalid transcript state fails soft
Given the transcript-state file is missing or contains invalid JSON,
when the TUI starts,
then Tau falls back to a clean transcript instead of crashing.

### AC-4 Restored sessions do not duplicate the welcome banner
Given a transcript was restored from local state,
when the TUI seeds startup messages,
then it does not append an extra copy of the welcome banner.

### AC-5 Earlier M335 REPL slices do not regress
Given runtime control, prompt history, transcript search/copy, and local
session persistence already exist,
when transcript persistence lands,
then those behaviors still pass in scoped regression coverage.

## Conformance Cases
- C-01 A saved local transcript snapshot restores prior messages into a new app.
  Maps to AC-1. Tier: Functional.
- C-02 After transcript mutations, Tau writes a JSON snapshot under
  `.tau/tui/interactive-transcript.json`. Maps to AC-2. Tier: Functional.
- C-03 Invalid or missing transcript-state files are ignored safely and the app
  starts clean. Maps to AC-3. Tier: Functional.
- C-04 Restored transcript startup does not duplicate the welcome system
  message. Maps to AC-4. Tier: Functional.
- C-05 Existing `#3677`, `#3678`, `#3679`, and `#3680` TUI regression tests
  continue to pass after transcript persistence lands. Maps to AC-5. Tier:
  Regression.

## Success Metrics / Observable Signals
- Prior transcript messages survive a restart
- Search and copy work against restored transcript messages
- Broken transcript-state files do not crash the TUI
- Restarting with a saved transcript does not prepend duplicate welcome lines

## Key Decisions
- Transcript persistence is local-only and file-backed under `.tau/tui/`
- Transcript state is stored separately from the smaller session snapshot so
  input edits do not rewrite large chat payloads on every keypress
- Transcript persistence follows transcript mutations rather than every input
  event
