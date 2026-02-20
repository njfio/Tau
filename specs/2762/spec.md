# Spec: Issue #2762 - Discord placeholder message + progressive edit streaming (G10)

Status: Implemented

## Problem Statement
Tau Discord outbound provider delivery posts final messages directly. G10 parity requires a streaming UX where Tau posts a placeholder message and progressively edits it as content accumulates.

## Acceptance Criteria

### AC-1 Discord provider delivery creates placeholder then progressively edits message
Given provider-mode outbound delivery for a Discord event with stream-compatible response length,
When delivery runs,
Then Tau first creates a placeholder Discord message and then PATCH-edits that message with progressively larger content until final text is complete.

### AC-2 Final progressive edit content equals full response text
Given a multi-segment response,
When progressive edits complete,
Then the last PATCH request body content equals the full response text.

### AC-3 Long Discord responses keep chunk-safe fallback behavior
Given response text exceeding Discord's safe 2000-char limit,
When provider delivery runs,
Then Tau continues using existing chunked POST behavior without progressive-edit path regressions.

### AC-4 Verification artifacts and checklist evidence are updated
Given implementation is complete,
When scoped quality gates and live validation run,
Then tests pass and `tasks/spacebot-comparison.md` updates G10 streaming evidence.

## Scope

### In Scope
- Provider-mode Discord outbound placeholder + PATCH progressive edit flow.
- Progressive edit tests for single and multi-segment responses.
- Regression test for >2000-char fallback behavior.
- Parity checklist evidence update.

### Out of Scope
- Serenity dependency/runtime migration.
- Discord thread creation API tooling.
- Telegram/WhatsApp progressive edit behavior.

## Conformance Cases
- C-01 (integration): provider Discord delivery posts placeholder then PATCHes final single-segment response.
- C-02 (integration): provider Discord delivery progressively PATCHes accumulated multi-segment content.
- C-03 (regression): >2000-char Discord response keeps chunked POST fallback.
- C-04 (verify/live): fmt, clippy, targeted tests, and local live validation pass.
- C-05 (docs): G10 streaming checklist row includes `#2762` evidence.

## Success Metrics / Observable Signals
- Discord users see one placeholder message that updates to final response instead of immediate final post.
- Progressive PATCH payloads show monotonically increasing content prefixes.
- Existing long-message safety constraints remain intact.
