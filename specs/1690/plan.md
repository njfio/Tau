# Issue 1690 Plan

Status: Reviewed

## Approach

1. Patch top-level module docs (`//!`) across selected files in:
   - `crates/tau-tools/src/`
   - `crates/tau-runtime/src/`
2. Focus comments on:
   - module responsibility boundaries
   - safety/policy integration contracts
   - reason-code/error-output expectations
3. Run targeted docs/contract tests to ensure no regressions.

## Affected Areas

- `crates/tau-tools/src/tool_policy_config.rs`
- `crates/tau-tools/src/mcp_server_runtime.rs`
- `crates/tau-tools/src/mcp_client_runtime.rs`
- `crates/tau-tools/src/tools/tool_policy.rs`
- `crates/tau-tools/src/tools/registry_core.rs`
- `crates/tau-runtime/src/runtime_output_runtime.rs`
- `crates/tau-runtime/src/transport_health.rs`
- `crates/tau-runtime/src/heartbeat_runtime.rs`
- `crates/tau-runtime/src/ssrf_guard.rs`
- `crates/tau-runtime/src/rpc_protocol_runtime.rs`

## Risks And Mitigations

- Risk: comments become generic noise
  - Mitigation: keep docs contract-focused (boundaries/policy/errors), not
    line-by-line narration.
- Risk: doc churn in large files causes merge friction
  - Mitigation: scope to file headers and minimal targeted additions.

## ADR

No architecture/dependency/protocol changes. ADR not required.
