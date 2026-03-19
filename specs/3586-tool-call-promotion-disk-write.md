# 3586 Textual Tool-Call Promotion And Disk-Write Proof

## Objective
Ensure Tau never leaves assistant-emitted JSON tool-call payloads as plain assistant text in the real runtime path, and prove with integration tests that natural-language requests can trigger real tool execution that writes to disk and runs shell commands.

## Inputs/Outputs
- Input: provider responses that may encode tool calls either as structured tool blocks or as textual JSON payloads in assistant text.
- Input: natural-language prompts that require file persistence or shell execution.
- Output: executable tool calls in the runtime loop, preserved tool results in session history, and observable hard failures when tool-call promotion cannot produce executable calls.
- Output: a real file written to disk through the runtime path for the write-tool scenario.

## Boundaries/Non-goals
- No TUI redesign, palette work, or transcript styling changes.
- No broad Tau persona rewrite beyond what is necessary to preserve existing tool-request semantics.
- No tool policy default changes.
- No silent fallback that converts promotion failure into plain assistant text.

## Failure Modes
- Provider returns assistant text containing `{"tool_calls": ...}` and runtime surfaces it as assistant text instead of executing it.
- Provider returns malformed textual tool-call payload and runtime silently continues as if the assistant answered normally.
- Natural-language write request completes without creating the target file.
- Natural-language bash request completes without recording the tool call and tool result in session history.
- Existing structured tool-call behavior regresses.

## Acceptance Criteria
- [ ] Textual JSON tool-call payloads from the provider are converted into executable tool calls before the runtime loop decides the assistant has produced a final plain-text answer.
- [ ] Malformed textual tool-call payloads fail loudly with observable error context instead of degrading into plain assistant text.
- [ ] A real integration test proves a natural-language prompt can trigger the `write` tool and persist a file on disk through the runtime path.
- [ ] A real integration test proves a natural-language prompt can trigger the `bash` tool and preserve both the tool call and tool result in session history.
- [ ] Existing structured tool-call tests remain green.

## Files To Touch
- `crates/tau-ai/src/openai.rs`
- `crates/tau-provider/src/codex_cli_client.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`
- `specs/3586-tool-call-promotion-disk-write.md`

## Error Semantics
- Promotion failure returns a typed invalid-response style error with machine-readable context indicating textual tool-call promotion failure.
- Runtime entrypoints may log/translate that error once, but interior parsing code must not silently swallow it.
- Successful promotion must preserve the tool name, tool arguments, and tool-call id when present.

## Test Plan
1. Add provider/parser red tests for textual JSON tool-call promotion from assistant text payloads.
2. Add provider/parser red tests for malformed textual tool-call payloads failing loudly.
3. Add runtime red integration test where a natural-language prompt causes a mocked provider response with textual `write` tool-call JSON and results in a real file on disk.
4. Add runtime red integration test where a natural-language prompt causes a mocked provider response with textual `bash` tool-call JSON and results in session history containing the tool call and tool result.
5. Implement minimal promotion/error changes.
6. Refactor duplicated parsing helpers without changing behavior.
7. Run targeted crates and integration tests for `tau-ai`, `tau-provider`, and `tau-coding-agent`.
