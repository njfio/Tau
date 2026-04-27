# Spec: Issue #3601 - Align CLI backend timeout with request timeout budget

Status: Reviewed

## Problem Statement
Local gateway/TUI runs can still fail with `codex cli timed out after 120000ms` even when the runtime is started with a larger `--request-timeout-ms` value. The provider layer currently configures Codex, Claude, and Gemini CLI backend subprocess timeouts from backend-specific flags only, so the CLI backend can time out before the caller's request timeout budget has expired.

## Scope
In scope:
- `crates/tau-provider/src/client.rs` CLI backend timeout selection.
- Codex, Claude, and Gemini CLI backend client construction.
- Provider tests proving request timeout budget alignment and larger explicit backend timeout preservation.
- `specs/3601/spec.md`, `specs/3601/plan.md`, and `specs/3601/tasks.md`.

Out of scope:
- HTTP provider timeout behavior.
- Retry budget semantics.
- TUI-only masking or banners without fixing the provider path.
- Gateway attempt timeout policy beyond the provider request budget already passed to provider client construction.

## Acceptance Criteria
### AC-1 Codex CLI backend honors the request timeout floor
Given OpenAI Codex CLI backend auth is selected,
when `request_timeout_ms` is larger than `openai_codex_timeout_ms`,
then the constructed Codex CLI client uses a timeout at least as large as the request timeout budget.

### AC-2 Claude CLI backend honors the request timeout floor
Given Anthropic Claude CLI backend auth is selected,
when `request_timeout_ms` is larger than `anthropic_claude_timeout_ms`,
then the constructed Claude CLI client uses a timeout at least as large as the request timeout budget.

### AC-3 Gemini CLI backend honors the request timeout floor
Given Google Gemini CLI backend auth is selected,
when `request_timeout_ms` is larger than `google_gemini_timeout_ms`,
then the constructed Gemini CLI client uses a timeout at least as large as the request timeout budget.

### AC-4 Explicit larger backend timeouts are preserved
Given a backend-specific CLI timeout is larger than `request_timeout_ms`,
when the provider client is built,
then the larger backend-specific timeout remains in effect.

### AC-5 Existing adapter behavior remains compatible
Given the existing Codex, Claude, and Gemini CLI adapter tests run,
when timeout budget alignment is applied,
then normal assistant text, textual tool-call promotion, isolated cwd behavior, and timeout failures remain green.

## Conformance Cases
- C-01 / AC-1 / Regression:
  a Codex CLI mock sleeps past the old backend timeout but below `request_timeout_ms`, and the provider completes successfully.
- C-02 / AC-2 / Regression:
  a unit-level builder/helper assertion proves Claude CLI timeout is floored by `request_timeout_ms`.
- C-03 / AC-3 / Regression:
  a unit-level builder/helper assertion proves Gemini CLI timeout is floored by `request_timeout_ms`.
- C-04 / AC-4 / Functional:
  backend-specific timeouts larger than the request timeout budget remain unchanged for all CLI backends.
- C-05 / AC-5 / Regression:
  existing CLI adapter filters stay green.

## Success Metrics / Observable Signals
- A local TUI/gateway request configured with a longer request timeout no longer fails early with `codex cli timed out after 120000ms` solely because the CLI backend kept the old default timeout.
- Provider timeout behavior is explainable from one rule: CLI backend timeout is `max(backend_specific_timeout_ms, request_timeout_ms)`.
- No Cargo dependency or lockfile changes are required.

## Files To Touch
- `specs/3601/spec.md`
- `specs/3601/plan.md`
- `specs/3601/tasks.md`
- `crates/tau-provider/src/client.rs`
- `crates/tau-provider/src/codex_cli_client.rs` only if integration coverage needs additional test-only access
