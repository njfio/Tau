# Issue 3799 Plan

## Approach

Wire the existing ops chat form handler into the tool-capable gateway agent
runtime. Keep the current POST/redirect flow, but initialize the selected
session as an agent conversation, register the configured gateway tools, run the
operator request, and persist the resulting user/tool/assistant messages back
into the same lineage. For action-oriented prompts, request a required tool
choice when actionable tools are available so the UI cannot satisfy a file or
runtime request with prose-only fallback text. When a tool result identifies a
workspace-local HTML artifact, load a bounded UTF-8 preview and render it in a
sandboxed chat canvas frame. For Agent Canvas v2, keep artifact history, inject
a sandbox-local diagnostics bridge, expose console/error/canvas-pixel signals to
the parent route, and send click/type/probe commands back into the frame through
postMessage.

## Affected Modules

- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_chat_canvas.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `scripts/dev/ops-chat-canvas-proof.sh`
- `scripts/dev/test-ops-chat-canvas-proof.sh`
- `specs/3799/*`
- `tasks/tau-gaps-issues-improvements.md`

## Risks

- Provider failures can make local-dev action requests fail instead of showing a
  deterministic fallback. Mitigation: return the gateway runtime error rather
  than pretending tools ran; browser validation must prove the happy path.
- Appending the assistant turn under the wrong parent could break lineage.
  Mitigation: reuse `initialize_gateway_session_runtime` and `persist_messages`
  so the agent's emitted message order is preserved.
- Embedding generated HTML could create parent-page script risk. Mitigation:
  render through `iframe sandbox="allow-scripts"` and only load canonical paths
  under the workspace root with a size cap.
- Canvas diagnostics could accidentally require parent access to sandboxed
  frame DOM. Mitigation: instrument the artifact inside the sandbox and report
  only bounded diagnostics through a named postMessage channel.
- UI/render hotspots could grow during feature work. Mitigation: move gateway
  artifact extraction and instrumentation into `ops_chat_canvas.rs` while
  preserving route-level render contracts.

## Verification

Run the targeted gateway tests first, then the touched crate checks and a live
Browser Use submission against the local `/ops/chat` route proving a file is
created through the UI. Run the new proof script test and live proof script
before the full workspace gate.
