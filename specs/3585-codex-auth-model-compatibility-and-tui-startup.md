# 3585 Codex Auth Model Compatibility And Tui Startup

## Objective
Make Tau fail loudly and early when Codex/ChatGPT-account auth is paired with an unsupported model, and make the local TUI startup path use a Codex-auth supported model by default so the tmux/just development loop works without manual model triage.

## Inputs/Outputs
- Inputs:
  - Tau CLI/runtime startup configuration
  - Codex-auth environment (`TAU_OPENAI_AUTH_MODE=oauth-token`)
  - requested model id
  - TUI/gateway requests sent through `/v1/responses`
- Outputs:
  - structured incompatibility error before long hanging turn states when the model/auth pair is invalid
  - supported default model for the local TUI startup path
  - TUI-visible operator message explaining the incompatibility and next action

## Boundaries/Non-goals
- No silent provider fallback.
- No redesign of unrelated TUI layout.
- No broad provider-auth refactor beyond Codex-auth compatibility handling.
- No weakening of hard-fail behavior.

## Failure Modes
- Runtime starts with Codex auth and an unsupported model.
- TUI enters `thinking` and never gets actionable model output.
- tmux/just startup path launches a runtime that is guaranteed to fail later.
- Compatibility error is swallowed or normalized into a generic timeout.

## Acceptance Criteria
- [ ] A Codex-auth + unsupported-model combination produces a structured hard failure before or during first request dispatch, with actionable text.
- [ ] The local dev/TUI startup path uses a Codex-auth supported model by default.
- [ ] The TUI transcript shows the incompatibility as a clear system/runtime error instead of an indefinite wait.
- [ ] The tmux/just startup path launches a runtime/TUI combination that can answer a simple prompt end-to-end under Codex auth.
- [ ] Tests cover unsupported-model rejection and supported-model startup/integration behavior.

## Files To Touch
- `crates/tau-cli/src/cli_args.rs`
- `crates/tau-provider/src/client.rs`
- `crates/tau-provider/src/codex_cli_client.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `crates/tau-tui/src/interactive/*`
- `justfile`
- related test files in the same crates

## Error Semantics
- Hard fail only.
- Structured error must preserve original provider message when available.
- Entrypoints may translate the error for display, but may not swallow or silently fallback.

## Test Plan
- Red unit test for Codex-auth unsupported-model validation.
- Red integration test for TUI/runtime surfacing the incompatibility.
- Green integration test for tmux/local startup using the supported model and answering a simple prompt.
- Regression test ensuring keep-alive SSE frames still do not break the TUI stream.
