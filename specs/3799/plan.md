# Issue 3799 Plan

## Approach

Wire the existing ops chat form handler into the existing Cortex completion
path. Keep the current POST/redirect flow, but after the user message is
persisted, request a Cortex response and append it as an assistant message under
the user message head. This keeps the UI simple and makes the live local route
truthful: a submitted chat message produces a persisted assistant turn.

## Affected Modules

- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/cortex_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3799/*`

## Risks

- Provider failures could make local-dev chat look broken. Mitigation: reuse
  Cortex fallback output, which already produces deterministic operator text on
  LLM errors.
- Appending the assistant turn under the wrong parent could break lineage.
  Mitigation: append the assistant message using the user message head returned
  by `SessionStore::append_messages`.

## Verification

Run the targeted gateway tests first, then the touched crate checks and a live
Browser Use submission against `http://127.0.0.1:8795/ops/chat`.
