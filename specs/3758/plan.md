# Plan: Issue #3758 - Supervised deploy/stop process lifecycle control

## Approach

1. Add RED tests around the existing deploy endpoint proving that a configured
   supervisor must spawn a real child process and stop must terminate it.
2. Introduce a gateway-local deploy process supervisor trait and command-backed
   implementation. The request body remains backward compatible; command
   selection is server-side configuration only.
3. Extend persisted deploy state and response payloads with process lifecycle
   fields.
4. Keep the existing gateway service-mode state transition as a compatibility
   signal, but make deploy/stop success depend on supervisor lifecycle success
   when a supervisor is configured.
5. Add/update specs and operator-facing documentation for the configured
   process boundary.
6. Close the adjacent P0/P1/P2 decision artifacts from the prioritized action
   list with docs and generated dependency graph evidence.
7. Run focused gateway tests and formatting/package checks.

## Affected Modules

- `crates/tau-gateway/src/gateway_openresponses/deploy_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/deploy_process_supervisor.rs`
- `crates/tau-gateway/src/gateway_openresponses/server_state.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests/state_helpers.rs`
- `CONTRIBUTING.md`
- `SECURITY.md`
- `docs/architecture/adr-006-dashboard-ui-stack.md`
- `docs/architecture/cortex-automation-scope.md`
- `docs/architecture/crate-dependency-diagram.md`
- `docs/guides/gateway-api-reference.md`
- `docs/guides/key-rotation-operator-runbook.md`
- `docs/guides/test-coverage-targets.md`
- `specs/milestones/m334/index.md`
- `tasks/reports/crate-dependency-graph.{json,md}`

## Risks / Mitigations

- Risk: allowing arbitrary deploy command execution through HTTP would be a
  remote-command surface.
  - Mitigation: do not add request-body command fields; supervisor command is
    configured server-side.
- Risk: existing clients may break if deploy now requires new request fields.
  - Mitigation: keep `{agent_id, profile, model}` sufficient.
- Risk: process handles are in-memory while metadata is persisted.
  - Mitigation: this slice supports same-runtime termination and durable
    evidence; lost-handle recovery can be a later process-pool slice.
- Risk: child-process tests can hang.
  - Mitigation: use short-lived shell sleep loops and always stop/abort handles
    in tests.
- Risk: docs imply Cortex can execute supervisor actions.
  - Mitigation: state advisory-only scope and defer supervisor-routing behind a
    future typed action contract.

## Interfaces / Contracts

- `GatewayDeployProcessSupervisor`
  - `start(request) -> GatewayDeployProcessStartResult`
  - `stop(agent_id, reason) -> GatewayDeployProcessStopResult`
- `CommandGatewayDeployProcessSupervisor`
  - configured with program/args by server/test setup, not request JSON.
- Deploy response gains additive process fields:
  - `process_id`
  - `process_status`
  - `process_pid`
  - `process_started_unix_ms`
- Stop response gains additive process fields:
  - `process_id`
  - `process_status`
  - `process_pid`
  - `process_stopped_unix_ms`
  - `process_stop_reason`
  - `process_exit_status`

Documentation contracts:

- Cortex is advisory-only in `docs/architecture/cortex-automation-scope.md`.
- Dashboard stack direction is Leptos SSR in
  `docs/architecture/adr-006-dashboard-ui-stack.md`.
- Quality thresholds live in `docs/guides/test-coverage-targets.md`.

## ADR

Not required for this slice. It adds an internal runtime abstraction and
backward-compatible response fields without new dependencies or external
protocol requirements.
