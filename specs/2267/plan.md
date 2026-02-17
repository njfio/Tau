# Plan #2267

Status: Reviewed
Spec: specs/2267/spec.md

## Approach

1. Add RED conformance tests first:
   - `tau-runtime`: deterministic fuzz loops over raw RPC envelope and NDJSON
     dispatch paths.
   - `tau-gateway`: deterministic fuzz loops over websocket request parse +
     classification helpers.
2. Add deterministic byte/JSON generator helpers in test modules (no external
   dependencies).
3. Add a local fuzz contract runner script that executes targeted conformance
   tests in a single command.
4. Add documentation for fuzz contract execution and scope.
5. Run scoped verification: fmt, clippy, targeted conformance tests, and full
   tests for touched crates.

## Affected Modules

- `crates/tau-runtime/src/rpc_protocol_runtime.rs`
- `crates/tau-gateway/src/gateway_ws_protocol.rs`
- `scripts/qa/*` (local fuzz contract runner)
- `docs/guides/*` (fuzz contract usage)
- `specs/2267/*`

## Risks and Mitigations

- Risk: high-iteration tests increase test runtime.
  - Mitigation: deterministic lightweight generators and bounded payload sizes.
- Risk: fuzz harness asserts too tightly and creates flaky tests.
  - Mitigation: assert only stability/no-panic/error-classification contract.
- Risk: partial coverage leaves important surfaces untested.
  - Mitigation: explicitly document current scope and follow-up expansion path.

## Interfaces / Contracts

- New local command contract:
  - `./scripts/qa/test-fuzz-contract.sh`
- Test contract:
  - deterministic >=10,000-iteration parser fuzz-conformance tests for specified
    runtime surfaces.
