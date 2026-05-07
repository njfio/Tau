# Issue 3799: Ops Chat Produces Assistant Replies

Status: Implemented
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The `/ops/chat` route renders a functional-looking chat composer, but submitting
the form only appends the operator message to session history. No assistant turn
is produced, so the surface behaves like a note log while presenting itself as a
chat interface.

## Scope

In scope:
- Make `/ops/chat/send` append a real assistant response after the user message.
- Reuse the existing Cortex chat completion/fallback path so local-dev works
  even when provider credentials are unavailable.
- Persist both user and assistant messages into the selected session lineage.
- Verify the live browser route submits and displays both turns.

Out of scope:
- Streaming the assistant response into the ops shell without a page reload.
- Adding new providers or production credentials.
- Reworking the broader chat UI layout.

## Acceptance Criteria

AC-1: Given an operator submits `/ops/chat/send`, when the form is accepted, then
the selected session contains both the user message and an assistant response.

AC-2: Given provider completion succeeds, when `/ops/chat/send` renders the
redirected `/ops/chat` page, then the transcript shows the assistant response
from the configured LLM client.

AC-3: Given provider completion fails in local-dev, when `/ops/chat/send` runs,
then the transcript shows the deterministic Cortex fallback response instead of
silently stopping after the user message.

AC-4: Given the live localhost route is tested in the in-app browser, when a
message is submitted, then the transcript count increases by two turns and
browser console errors are zero.

## Conformance Cases

C-01 maps to AC-1 and AC-2: gateway integration test posts
`/ops/chat/send` with a scripted LLM response and asserts user plus assistant
rows are rendered and persisted.

C-02 maps to AC-3: existing Cortex fallback behavior is reused by the ops chat
handler so local-dev provider errors still produce an assistant row.

C-03 maps to AC-4: Browser Use live validation submits the ops chat form and
asserts the transcript includes the user message and an assistant response.

## Success Signals

- `cargo test -p tau-gateway integration_spec_3799_c01_ops_chat_send_appends_assistant_reply`
- `cargo test -p tau-gateway gateway_openresponses::tests::integration_spec_2830_c02_c03_ops_chat_send_appends_message_and_renders_transcript_row`
- Browser Use confirms `/ops/chat` appends two turns and reports zero console
  errors.
