# Issue 3799 Plan

## Approach

Wire the existing ops chat form handler into the tool-capable gateway agent
runtime. Keep the current POST/redirect flow, but initialize the selected
session as an agent conversation, register the configured gateway tools, run the
operator request, and persist the resulting user/tool/assistant messages back
into the same lineage. For action-oriented prompts, request a required tool
choice when actionable tools are available so the UI cannot satisfy a file or
runtime request with prose-only fallback text.

## Affected Modules

- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3799/*`

## Risks

- Provider failures can make local-dev action requests fail instead of showing a
  deterministic fallback. Mitigation: return the gateway runtime error rather
  than pretending tools ran; browser validation must prove the happy path.
- Appending the assistant turn under the wrong parent could break lineage.
  Mitigation: reuse `initialize_gateway_session_runtime` and `persist_messages`
  so the agent's emitted message order is preserved.

## Verification

Run the targeted gateway tests first, then the touched crate checks and a live
Browser Use submission against the local `/ops/chat` route proving a file is
created through the UI.
