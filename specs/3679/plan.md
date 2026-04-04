# Plan: Issue #3679 - Add transcript search, copy, and stronger scrollback to `tau-tui` REPL

Status: Implemented

## Approach
Extend the in-memory chat panel with local search state, add transcript copy
helpers that reuse the existing clipboard integration, and bind page-style chat
navigation in normal mode. Keep the slice entirely inside `tau-tui`.

## Affected Modules
- `crates/tau-tui/src/interactive/chat.rs`
  - add search state, search navigation, and search summary helpers
- `crates/tau-tui/src/interactive/app.rs`
  - add transcript search command handlers and search-state messaging
- `crates/tau-tui/src/interactive/app_commands.rs`
  - route `/search`, `/search-next`, `/search-prev`, `/search-clear`, and
    `/copy-last`
- `crates/tau-tui/src/interactive/app_keys.rs`
  - add `PageUp` / `PageDown` chat navigation
- `crates/tau-tui/src/interactive/app_copy_target.rs`
  - reuse clipboard plumbing for transcript-copy helpers
- `crates/tau-tui/src/interactive/ui_chat.rs`
  - render search summary above the transcript/tool summary area
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - document search/copy/scrollback controls
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/green tests for search, copy, and scrollback

## Contracts
- Search is case-insensitive and scoped to in-memory chat messages only
- Search commands do not mutate the transcript itself
- `/copy-last` defaults to the latest chat entry and may optionally filter by
  role
- `PageUp` / `PageDown` apply only when the chat panel is focused in normal mode

## Risks
- Search state must stay consistent as new messages append to the transcript
- Copy helpers should handle empty transcripts and missing role matches
- The new search summary should not crowd out the existing tool/build summary

## Verification Strategy
- Add failing transcript tests first for search, copy, and scrollback
- Re-run the broader `interactive::app_gateway_tests` regression suite
- Build `tau-tui` after the scoped tests pass
