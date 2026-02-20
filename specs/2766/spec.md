# Spec: Issue #2766 - Discord thread creation command and provider typing dispatch (G10)

Status: Implemented

## Problem Statement
Tau's Discord adapter coverage is missing two operator-visible capabilities: explicit thread creation dispatch and concrete provider typing indicator emission. This keeps the G10 implementation row open despite existing send/receive/file/reaction support.

## Acceptance Criteria

### AC-1 Outbound supports Discord thread creation requests
Given a Discord inbound event and thread name,
When thread creation is dispatched,
Then outbound provider mode POSTs to Discord thread-creation endpoint with the expected payload and returns provider identifiers in receipts.

### AC-2 Runtime `/tau thread` command dispatches thread creation without assistant text reply
Given an inbound `/tau thread <name>` command event,
When runtime processes the event in outbound-enabled mode,
Then runtime executes thread dispatch, logs command metadata and thread delivery payload, and suppresses normal assistant text output.

### AC-3 Runtime emits Discord provider typing indicator dispatch when typing lifecycle is enabled
Given Discord provider mode and typing lifecycle emission conditions are met,
When runtime persists an event,
Then runtime dispatches Discord typing indicator and records typing delivery payload/log context.

### AC-4 Regression safety preserves existing non-Discord behavior
Given Telegram or WhatsApp transports and existing command flows,
When this feature is enabled,
Then prior behavior remains unchanged and existing tests continue to pass.

### AC-5 Verification artifacts and G10 checklist evidence are updated
Given implementation is complete,
When scoped quality gates and live validation run,
Then tests pass and `tasks/spacebot-comparison.md` marks the G10 implementation row with issue evidence.

## Scope

### In Scope
- `tau-multi-channel` outbound methods for Discord thread and typing dispatch.
- Runtime command parser/execution/logging integration for `/tau thread`.
- Runtime typing-lifecycle integration with provider dispatch.
- Conformance/regression tests and checklist updates.

### Out of Scope
- `serenity` dependency migration.
- New external services or protocol/schema changes.
- Full Discord runtime crate extraction.

## Conformance Cases
- C-01 (integration): Discord outbound thread creation request shape/endpoint and receipt metadata.
- C-02 (functional): runtime `/tau thread` command path records command + thread delivery and suppresses assistant text response.
- C-03 (integration): runtime/provider Discord typing indicator dispatch occurs when lifecycle emits typing.
- C-04 (regression): non-Discord command flows remain green.
- C-05 (verify/live): fmt, clippy, targeted tests, and localhost live validation pass.
- C-06 (docs): G10 implementation checklist row includes `#2766` evidence.

## Success Metrics / Observable Signals
- Operators can initiate Discord thread creation from command flow.
- Discord provider typing endpoint is exercised during lifecycle emission.
- G10 implementation row is closed with linked issue evidence.
