# Spec: Issue #3584 - Redesign Webchat as a Production Operator Console

Issue: https://github.com/njfio/Tau/issues/3584
Status: Planned

## Objective
Redesign Tau webchat into a production-grade operator console that is genuinely useful to end users and the wider community. The web application must provide a coherent information architecture for conversation, tasks, tools, artifacts, memory, cortex, routines, sessions, and share/replay workflows while consuming the same shared operator state protocol as the TUI.

## Inputs/Outputs
- Inputs:
  - Shared operator turn/task state protocol from Issue #3581.
  - Runtime and gateway control actions, approvals, interrupts, retries, and session selection.
  - Artifact, memory, cortex, and routine/job state from the unified runtime.
- Outputs:
  - A web application with production operator UX for chat, task progress, tools, artifacts, memory, cortex, routines, and sessions.
  - Shareable and replayable run/session views suitable for community/demo usage.
  - A modern design system and interaction model rather than a debug/control page.

## Boundaries / Non-goals
- No TUI redesign in this issue.
- No independent web-only state model; webchat must consume the shared contract.
- No narrow endpoint-by-endpoint cleanup without product-level IA and interaction changes.
- No swarm orchestration implementation beyond the web hooks/navigation the console depends on.

## Failure Modes
1. Long-running turn with tools, partial output, and artifacts.
   - Expected: transcript, task progress, and artifact visibility remain coherent and readable.
2. Approval-required or interrupted turn.
   - Expected: operator actions are obvious, safe, and reflected live in the UI.
3. Runtime/provider failure.
   - Expected: failure state includes actionable context and recovery actions rather than raw JSON/debug dump text.
4. Memory/cortex/routines/tools state drifts from conversation state.
   - Expected: parity tests fail; UI must reflect one coherent session/turn state.
5. Community/share use case.
   - Expected: a run/session can be inspected or replayed without exposing debug-only internal clutter as the main UX.

## Acceptance Criteria (testable booleans)
- [ ] AC-1: Webchat information architecture defines core operator views for chat, tasks, tools, artifacts, memory, cortex, routines, sessions, and share/replay.
- [ ] AC-2: Prompt submission, streaming, approvals, interrupts, retries, and recovery actions are defined as first-class workflows.
- [ ] AC-3: Webchat consumes the shared state protocol from Issue #3581 instead of relying primarily on endpoint-specific raw JSON payload rendering.
- [ ] AC-4: Visual/system design is specified for a modern community-facing operator product rather than a diagnostics page.
- [ ] AC-5: Core workflows have browser test coverage proving pages/views actually work end to end.
- [ ] AC-6: Webchat/dashboard parity with runtime and TUI is covered for core turn/task/tool/memory/artifact states.

## Files To Touch
- `crates/tau-gateway/src/gateway_openresponses/webchat_page.rs`
- `crates/tau-gateway/src/gateway_openresponses/stream_response_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/ws_stream_handlers.rs`
- `crates/tau-gateway/src/gateway_openresponses/types.rs`
- `crates/tau-gateway/src/gateway_openresponses/server_state.rs`
- `crates/tau-gateway/src/gateway_openresponses/status_runtime.rs`
- `scripts/verify/unified-web-ui-live-smoke.mjs`
- `scripts/verify/unified-web-ui-live-smoke.spec.js`
- `scripts/verify/test-unified-web-ui-live-smoke.sh`
- `docs/guides/operator-deployment-guide.md`

## Error Semantics
- Failed streaming, runtime/provider failure, or control action failure must be rendered as explicit operator-visible state with actionable context.
- Webchat may summarize structured state for UX, but must not hide or silently coerce failure semantics.
- If a dependent subsystem is unavailable during migration, the UI must render explicit degraded state rather than an empty successful-looking panel.

## Test Plan
1. Add browser tests covering conversation, task progress, approvals, interrupts, retries, artifacts, sessions, memory, cortex, and routines.
2. Add parity tests proving the same turn/task scenario renders equivalent state in webchat and TUI.
3. Add live-smoke verification through unified startup proving all webchat views load and update from shared runtime state.
4. Add regression tests preventing fallback to raw JSON/debug-dump-first rendering for primary operator workflows.

## Rollout Notes
1. Land shared state contract first.
2. Build the webchat app shell around shared transcript/task/artifact state.
3. Migrate secondary views (memory, cortex, routines, sessions, tools) onto the same state model.
4. Add share/replay surfaces once the core operator workflows are stable.
