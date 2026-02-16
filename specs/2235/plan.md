# Plan #2235

Status: Implemented
Spec: specs/2235/spec.md

## Approach

1. Add OpenAI endpoint helpers:
   - `chat_completions_url()`
   - `responses_url()`
2. Add Codex routing heuristic:
   - route models containing `codex` to Responses API directly.
3. Add fallback behavior:
   - when chat-completions returns a known model-not-supported message for
     Codex-only models, retry via Responses API once.
4. Add Responses API payload support:
   - build request body with `model`, `input`, and optional controls.
   - parse top-level `status`, `usage`, and `output` message text blocks.
5. Keep non-Codex flow unchanged:
   - existing chat-completions path and stream parsing remain default for
     non-Codex models.
6. Validate with tests and live request:
   - RED tests for routing/parsing/fallback.
   - GREEN implementation.
   - run `cargo test -p tau-ai`.

## Affected Modules

- `crates/tau-ai/src/openai.rs`
- `crates/tau-ai/tests/openai_http_e2e.rs` (if integration additions needed)

## Risks and Mitigations

- Risk: False-positive routing of non-Codex models.
  - Mitigation: strict model-id check and regression tests.
- Risk: Responses payload variation (string vs structured content).
  - Mitigation: tolerant parser that extracts `output_text` and text blocks.
- Risk: behavior regressions in chat-completions.
  - Mitigation: preserve existing path and add regression conformance case.

## Interfaces/Contracts

- No public trait changes (`LlmClient` unchanged).
- `OpenAiClient` internals gain Responses API path and parser utilities only.
