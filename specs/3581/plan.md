# Plan: Issue #3581 - Define shared operator turn/task state protocol for TUI and webchat

## Approach
1. Capture the shared-state contract in spec artifacts and an ADR before changing code.
2. Add RED contract tests in `tau-contract` that describe the schema, fixture parsing, and phase/status vocabulary.
3. Implement additive `OperatorTurnState` v1 types and fixtures in `tau-contract` without changing existing gateway/TUI/webchat runtime behavior.
4. Document runtime mappings and client consumption rules in `docs/architecture/shared-operator-state-v1.md`.
5. Run scoped contract tests, clippy, full formatting, and root manifest drift gates before committing and closing #3581.

## Current Surface Map
- `crates/tau-agent-core/src/lib.rs`: defines `AgentEvent`, including turn, message, tool, safety, cost, and replan events that can feed operator state.
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`: maps runtime events into gateway streaming behavior and includes `response.tool_execution.started` / `response.tool_execution.completed` event semantics.
- `crates/tau-gateway/src/gateway_openresponses/stream_response_handler.rs`: owns streaming response completion and failure framing.
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`: models mission states such as running, completed, checkpointed, and blocked.
- `crates/tau-gateway/src/gateway_openresponses/mission_api_runtime.rs`: exposes mission list/detail snapshots consumed by clients.
- `crates/tau-tui/src/interactive/gateway_client.rs`: parses gateway SSE into local TUI events.
- `crates/tau-tui/src/interactive/app.rs`: keeps local pending turn, streaming assistant index, and tool reconciliation state.
- `crates/tau-gateway/src/gateway_openresponses/webchat_page.rs`: parses response SSE in embedded webchat JavaScript.
- `crates/tau-gateway/src/gateway_openresponses/dashboard_runtime.rs`: emits snapshot-oriented dashboard state.
- `crates/tau-contract/src/lib.rs`: existing shared fixture/contract helper crate and preferred additive schema home for this stage.

## ADR Trigger
This stage defines a shared operator wire/schema contract across gateway, TUI, webchat, dashboard, and future runtime surfaces. That is a cross-cutting architecture decision. Record it in `docs/adrs/0005-shared-operator-turn-task-state.md` before implementing types.

## Proposed Contract Shape
The first additive schema should be intentionally small:
- `OperatorTurnState`
- `OperatorTurnEvent`
- `OperatorTurnPhase`
- `OperatorTurnStatus`
- `OperatorToolState`
- `OperatorErrorContext`
- `OperatorTurnFixture`

Minimum vocabulary:
- Phases: `created`, `queued`, `running`, `waiting_for_tool`, `waiting_for_verifier`, `completed`.
- Statuses: `pending`, `streaming`, `tool_running`, `blocked`, `timed_out`, `failed`, `succeeded`, `cancelled`.
- Event kinds: text delta, tool started, tool completed, tool failed, checkpointed, blocked, timeout, final answer.

## Backwards Compatibility
Preserve all existing event and endpoint surfaces during this definition stage:
- `response.output_text.delta`
- `response.tool_execution.started`
- `response.tool_execution.completed`
- `response.completed`
- `response.failed`
- `/gateway/missions`
- `/gateway/missions/{mission_id}`
- dashboard stream/status endpoints

## Compatibility Assessment
```yaml
implementation_strategy:
  task: "3581"
  change_surface:
    - symbol: "OperatorTurnState v1"
      location: "crates/tau-contract/src/lib.rs"
      change_type: "additive shared contract"
      current: "no shared TUI/webchat operator turn state contract"
      proposed: "serde-serializable schema and fixtures"
      compatibility: "backwards-compatible"
      reason: "existing clients continue to read legacy gateway SSE and mission/dashboard surfaces"
  overall_compatibility: "safe"
  version_impact: "minor-internal"
```

## Risks / Mitigations
- Risk: contract is too broad and becomes a second runtime model.
  Mitigation: keep v1 additive and fixture-oriented; map existing surfaces before wiring clients.
- Risk: TUI/webchat migration changes legacy SSE behavior.
  Mitigation: do not remove or rename existing `response.*` events in this stage.
- Risk: new internal dependency churn.
  Mitigation: use existing `tau-contract` crate and existing serde dependencies; no root manifest or lockfile drift.
- Risk: schema fields become ambiguous.
  Mitigation: write examples for success, tool failure, timeout, and blocked mission in docs and tests.

## Verification
- `cargo test -p tau-contract operator_state -- --test-threads=1`
- `cargo clippy -p tau-contract --tests --no-deps -- -D warnings`
- `cargo fmt --check`
- `git diff --quiet -- Cargo.toml Cargo.lock`
