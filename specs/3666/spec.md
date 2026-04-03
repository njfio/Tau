# Spec: Issue #3666 - Teach CLI provider adapters to emit textual tool-call payloads

Status: Implemented

## Problem Statement
Tau's CLI-backed provider adapters currently tell the model to return plain
assistant text only and describe tools as context only. In gateway Ralph-loop
flows this causes tool-capable tasks to fabricate natural-language completion
instead of emitting the textual `tool_calls` payloads that Tau can promote into
structured tool execution.

## Scope
In scope:
- `crates/tau-provider/src/codex_cli_client.rs`
- `crates/tau-provider/src/claude_cli_client.rs`
- `crates/tau-provider/src/gemini_cli_client.rs`
- `specs/3666/spec.md`
- `specs/3666/plan.md`
- `specs/3666/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- changing HTTP-native provider tool wiring
- changing gateway verifier or retry policy
- redesigning the textual tool-call promotion parser

## Acceptance Criteria
### AC-1 CLI adapters explicitly teach textual tool-call emission
Given a CLI-backed provider request includes tools,
when the adapter renders the model prompt,
then it explicitly instructs the model to return textual `tool_calls` JSON when
it needs a tool instead of describing the action in prose.

### AC-2 Codex CLI path proves promoted textual tool-call execution contract
Given the Codex CLI adapter receives a tool-enabled request,
when a provider script follows the rendered contract and returns textual
`tool_calls` JSON,
then Tau promotes that output into structured tool calls.

### AC-3 Prompt contract stays aligned across Codex, Claude, and Gemini CLI adapters
Given the three CLI adapters render tool-enabled prompts,
when their prompt contracts are inspected in tests,
then they all expose the same textual tool-call guidance instead of the old
context-only wording.

### AC-4 Plain text non-tool responses remain supported
Given a CLI-backed provider request does not need a tool,
when the adapter completes normally,
then plain assistant text responses still parse successfully.

## Conformance Cases
- C-01 / AC-1 / Functional:
  Codex CLI tool-enabled prompt includes explicit textual `tool_calls` JSON
  guidance and exact-tool-name instructions.
- C-02 / AC-2 / Regression:
  Codex CLI mock script returns textual tool-call JSON and the adapter promotes
  it into a structured tool call.
- C-03 / AC-3 / Functional:
  Claude CLI tool-enabled prompt includes the same textual tool-call guidance.
- C-04 / AC-3 / Functional:
  Gemini CLI tool-enabled prompt includes the same textual tool-call guidance.
- C-05 / AC-4 / Regression:
  existing plain-text parsing behavior remains green.

## Success Metrics / Observable Signals
- Tool-capable CLI provider turns stop narrating fake filesystem work and start
- producing executable textual tool-call payloads that the agent runtime can run.
- Gateway Ralph-loop retries can observe real `ToolExecutionStart` events on
  CLI-backed model paths.

## Files To Touch
- `specs/3666/spec.md`
- `specs/3666/plan.md`
- `specs/3666/tasks.md`
- `specs/milestones/m334/index.md`
- `crates/tau-provider/src/codex_cli_client.rs`
- `crates/tau-provider/src/claude_cli_client.rs`
- `crates/tau-provider/src/gemini_cli_client.rs`
