# Spec #2524 - Story: expose send_file intent via built-in tool contract

Status: Implemented

## Problem Statement
Current send-file behavior is command-driven; story-level parity needs built-in tool-call semantics for deterministic agent intent.

## Acceptance Criteria
### AC-1
Given a `send_file` tool call, when executed, then result payload includes file path metadata and stable reason code.

### AC-2
Given a successful send-file tool result in a run, when turn finalizes, then no follow-up text is emitted.

### AC-3
Given send-file-only run diagnostics, when outbound payload is written, then file metadata is present.

## Scope
In scope:
- Tool contract and suppression semantics.
- Outbound diagnostics for observability.

Out of scope:
- New adapter transports.

## Conformance Cases
- C-01: `spec_2525_c02_send_file_tool_returns_structured_file_delivery_payload`
- C-02: `integration_spec_2525_c03_prompt_send_file_tool_call_terminates_run_without_follow_up_model_turn`
- C-03: `integration_spec_2525_c05_runner_persists_send_file_payload_and_suppresses_text_reply`
