# Spec: Issue #3585 - Codex auth runtime rejects unsupported models and stops TUI hangs

Status: Reviewed

## Problem Statement
When Tau is started with Codex/ChatGPT-account auth and an incompatible model such as `openai/gpt-5.2`, interactive gateway/TUI requests can remain in long thinking states and eventually surface generic timeout errors. Direct `codex exec` reports a clear model/auth incompatibility, so Tau should fail closed with actionable text before operators have to infer the mismatch from a hang.

## Scope
In scope:
- `crates/tau-provider` Codex auth/provider model compatibility checks.
- `crates/tau-tui` interactive error propagation for this incompatibility.
- Local/dev startup guidance or tests for a supported Codex-auth model path.
- `specs/3585/spec.md`, `specs/3585/plan.md`, and `specs/3585/tasks.md`.

Out of scope:
- Reworking the whole provider stack.
- Adding silent provider/model fallbacks.
- General TUI visual redesign unrelated to runtime/auth failure handling.
- Changing supported API-key OpenAI behavior.

## Acceptance Criteria
### AC-1 Unsupported Codex-auth models fail closed with actionable errors
Given Tau is configured to use Codex/ChatGPT-account auth with an unsupported model such as `openai/gpt-5.2`,
when a provider request is constructed or dispatched,
then Tau returns a structured hard failure that names the model/auth incompatibility and does not wait for the normal request timeout budget.

### AC-2 Supported Codex-auth model startup remains green
Given the local TUI/dev startup path uses Codex auth,
when the configured model is supported by the Codex backend,
then the runtime path remains accepted and no unsupported-model error is emitted.

### AC-3 TUI surfaces the incompatibility in-session
Given the provider reports an unsupported Codex-auth model error,
when the TUI receives the gateway/runtime failure,
then it displays actionable text that tells the operator to select a supported Codex-auth model or change auth mode.

### AC-4 Existing Codex provider behavior remains compatible
Given existing Codex CLI/provider regressions run,
when unsupported-model validation is added,
then normal assistant text, textual tool-call promotion, and timeout/error parsing behavior remain green for supported configurations.

## Conformance Cases
- C-01 / AC-1 / Regression:
  a Codex-auth provider configuration with `openai/gpt-5.2` returns an immediate unsupported-model error containing the model name and auth mode.
- C-02 / AC-2 / Functional:
  a Codex-auth provider configuration with a supported Codex model passes validation and can reach the existing mock provider execution path.
- C-03 / AC-3 / Regression:
  a TUI-facing failure conversion/rendering test includes the actionable unsupported-model guidance text.
- C-04 / AC-4 / Regression:
  existing `codex_cli_client` tests stay green after the validation change.

## Success Metrics / Observable Signals
- Interactive Codex-auth/TUI misconfiguration fails quickly instead of hanging until a generic timeout.
- Error text contains enough detail for an operator to fix configuration without reading provider source code.
- Supported-model tests and existing Codex provider adapter tests remain green.

## Files To Touch
- `specs/3585/spec.md`
- `specs/3585/plan.md`
- `specs/3585/tasks.md`
- `crates/tau-provider/src/*codex*`
- `crates/tau-tui/src/interactive/*` as needed for in-session error text
