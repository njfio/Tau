## Objective

Ensure CLI-backed provider timeouts do not undercut the runtime request timeout budget, so a runtime started with a larger `request_timeout_ms` does not still fail early inside the Codex/Claude/Gemini CLI subprocess layer.

## Inputs/Outputs

- Inputs:
  - `Cli.request_timeout_ms`
  - Provider-specific CLI timeout flags:
    - `openai_codex_timeout_ms`
    - `anthropic_claude_timeout_ms`
    - `google_gemini_timeout_ms`
- Outputs:
  - Effective CLI backend timeout used by provider client construction
  - Regression coverage for the effective-timeout rule

## Boundaries/Non-goals

- Only CLI-backed provider clients are in scope.
- HTTP provider timeout behavior is out of scope.
- Retry policy behavior is out of scope.
- No TUI-only masking or message changes without fixing provider timeout propagation.

## Failure modes

- A CLI backend timeout lower than `request_timeout_ms` causes invalid early failures such as `codex cli timed out after 120000ms` even though the runtime requested a larger timeout.
- Changing timeout precedence incorrectly could break explicit longer backend timeout overrides.

## Acceptance criteria

- [ ] OpenAI Codex CLI timeout is `max(openai_codex_timeout_ms, request_timeout_ms)`.
- [ ] Claude CLI timeout is `max(anthropic_claude_timeout_ms, request_timeout_ms)`.
- [ ] Gemini CLI timeout is `max(google_gemini_timeout_ms, request_timeout_ms)`.
- [ ] Larger backend-specific timeout values remain preserved.
- [ ] A regression test proves the OpenAI Codex backend succeeds when `request_timeout_ms` exceeds the old backend timeout and the mock CLI sleeps past the old timeout.

## Files to touch

- `specs/3601-cli-backend-timeout-aligns-with-request-timeout-budget.md`
- `crates/tau-provider/src/client.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/auth_and_provider/provider_client_and_store.rs`

## Error semantics

- Provider timeout failures remain hard-fail errors.
- This change only adjusts timeout selection; it must not introduce fallback behavior or silent retries.

## Test plan

- Add unit tests in `crates/tau-provider/src/client.rs` for the effective CLI timeout helper.
- Add a regression test in `crates/tau-coding-agent/src/tests/auth_provider/auth_and_provider/provider_client_and_store.rs`:
  - configure `request_timeout_ms` larger than `openai_codex_timeout_ms`
  - use a mock Codex CLI script that sleeps past the old smaller timeout but completes within the request timeout
  - assert the provider client succeeds
- Run targeted provider and runtime auth/provider tests.
