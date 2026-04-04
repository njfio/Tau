# Plan: Issue #3681 - Add local transcript persistence to `tau-tui` REPL

Status: Implemented

## Approach
Add a separate file-backed transcript snapshot for the interactive TUI and
restore it before startup banner seeding. Keep transcript persistence local to
`tau-tui`, and persist only when the transcript changes so normal input edits
do not rewrite large transcript files.

## Affected Modules
- `crates/tau-tui/src/interactive/chat.rs`
  - make persisted chat messages serializable and add restore helpers if needed
- `crates/tau-tui/src/interactive/app.rs`
  - expose transcript snapshot/restore helpers and track transcript mutations
- `crates/tau-tui/src/interactive/app_commands.rs`
  - route transcript-clearing through an app helper so persistence can observe
    the mutation
- `crates/tau-tui/src/interactive/app_runtime.rs`
  - load transcript state before seeding welcome messages and persist when the
    transcript revision changes
- `crates/tau-tui/src/interactive/mod.rs`
  - add transcript-state modules
- new `crates/tau-tui/src/interactive/transcript_state.rs`
  - file-backed transcript snapshot type and read/write helpers
- new `crates/tau-tui/src/interactive/transcript_state_tests.rs`
  - RED/green transcript persistence tests

## Contracts
- Default local transcript path: `.tau/tui/interactive-transcript.json`
- Persisted fields:
  - transcript schema version
  - chat messages with role/content/timestamp
- Missing or invalid transcript state returns a clean empty transcript
- Restored transcript state must not trigger gateway requests

## Risks
- Persisting large transcripts too often can create unnecessary I/O; persist on
  transcript revision changes only
- Chat role serialization should remain stable enough for local durability
- Startup should avoid welcome-banner duplication when transcript history exists

## Verification Strategy
- Add failing tests first for transcript restore, save, invalid-state fallback,
  and welcome-banner deduplication
- Re-run existing `interactive::app_gateway_tests`
- Build `tau-tui` after the scoped tests pass
