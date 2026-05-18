# Issue 3799: Ops Chat Produces Assistant Replies

Status: Implemented
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The `/ops/chat` route renders a functional-looking chat composer, but submitting
the form must be truthful to the operator's intent. It cannot stop at appending
messages or returning advisory Cortex fallback text when the request asks Tau to
create, edit, validate, or run work. Action-oriented sends need to execute the
registered gateway tools and persist that evidence back into the transcript.

## Scope

In scope:
- Make `/ops/chat/send` append a real assistant response after the user message.
- Route accepted sends through the tool-capable agent runtime with the existing
  gateway tool registrar.
- Persist user, tool, and assistant messages into the selected session lineage.
- Record observer metadata that distinguishes direct assistant replies from
  tool-backed turns.
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

AC-3: Given an operator asks `/ops/chat/send` to create, edit, validate, or run
work, when registered gateway tools are available, then the handler requires a
tool-capable model turn, executes the tool call, and persists the tool result in
the transcript before the assistant reply.

AC-4: Given the live localhost route is tested in the in-app browser, when a
tool-backed message is submitted, then the transcript shows the operator
message, a tool result row, and the final assistant reply.

## Conformance Cases

C-01 maps to AC-1 and AC-2: gateway integration test posts
`/ops/chat/send` with a scripted LLM response and asserts user plus assistant
rows are rendered and persisted.

C-02 maps to AC-3: gateway integration test posts an action-oriented chat
message with a scripted `write` tool call and asserts the file is created, the
tool row is rendered, and observer metadata reports tool execution.

C-03 maps to AC-4: Browser Use live validation submits the ops chat form and
asserts the transcript includes the user message, rendered tool result, and
assistant response, with the requested file present on disk.

## Success Signals

- `cargo test -p tau-gateway integration_spec_3799_c01_ops_chat_send_appends_assistant_reply`
- `cargo test -p tau-gateway integration_spec_3799_c04_ops_chat_send_executes_registered_tools_for_action_requests`
- `cargo test -p tau-gateway gateway_openresponses::tests::integration_spec_2830_c02_c03_ops_chat_send_appends_message_and_renders_transcript_row`
- Browser Use confirms `/ops/chat` appends user, tool, and assistant rows and
  creates the requested file on disk.
