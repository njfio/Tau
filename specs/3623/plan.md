# Plan: Issue #3623 - Close agent runtime integrity gaps across persistence, telemetry, MCP, skills, and orchestration

Status: Implemented

Milestone: specs/milestones/m329/index.md

## Approach
Deliver the work in four slices so behavior changes stay reviewable and testable
without mixing unrelated runtime surfaces in one PR.

### Slice 1 - Action-history persistence + telemetry fidelity
1. Add failing coverage proving `prompt*` paths do not persist action history
   while `continue_turn*` does.
2. Refactor `tau-agent-core` so both entrypoint families use one action-history
   finalization path.
3. Thread real turn numbers and measured tool latency into action-history
   recording at the tool execution boundary.
4. Verify persisted history round-trips through existing `tau-memory`
   reporting/failure-pattern APIs.

### Slice 2 - MCP tool honesty/runtime effect alignment
1. Add failing coverage for `tau.training_trigger` and `tau.agent_*` returning
   success-shaped responses without runtime effects.
2. For each tool, choose one explicit outcome:
   - runtime-backed implementation with testable side effects, or
   - explicit not-implemented/error contract that stops pretending to succeed.
3. Update tool descriptions/docs so discovery text matches actual behavior.

### Slice 3 - MCP skills parity with `tau-skills`
1. Add failing coverage showing MCP skills behavior diverges from
   `tau-skills` manifest/trust/install flows.
2. Replace the ad hoc filesystem-only handlers in `tau-tools` with adapters
   around `tau-skills` catalog/install/runtime APIs.
3. Preserve the MCP UX shape where practical, but source all metadata and
   install guarantees from `tau-skills`.

### Slice 4 - `plan_executor` truth-in-advertising
1. Decide explicitly whether this issue will:
   - narrow docs/API language to match the current reporting/deadlock role, or
   - implement real execution/scheduling.
2. Unless product requirements force executor implementation, prefer the
   documentation/API correction path in this issue and track real execution as a
   follow-up.
3. Add targeted tests that prevent the module from drifting back into
   aspirational language without matching runtime behavior.

## Affected Modules
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-memory/src/action_history.rs`
- `tests/integration/tests/agent_tool_memory_roundtrip.rs`
- `crates/tau-tools/src/mcp_server_runtime.rs`
- `crates/tau-skills/src/lib.rs`
- `crates/tau-skills/src/skill_runtime.rs`
- `crates/tau-orchestrator/src/plan_executor.rs`
- `crates/tau-orchestrator/src/orchestrator.rs`
- `docs/guides/multi-agent-ops.md`

## Test Plan
- `cargo test -p tau-agent-core`
- `cargo test -p tau-tools`
- `cargo test -p tau-skills`
- `cargo test -p tau-orchestrator`
- `cargo test --test agent_tool_memory_roundtrip` or equivalent targeted
  integration selection if the workspace layout requires it
- `cargo fmt --check`
- `cargo clippy -- -D warnings`

## Verification Results
- `rustfmt --check --edition 2021 crates/tau-agent-core/src/lib.rs crates/tau-agent-core/src/tests/action_history.rs crates/tau-tools/src/mcp_server_runtime.rs crates/tau-ops/src/mcp_sdk.rs crates/tau-orchestrator/src/plan_executor.rs`
- `CARGO_TARGET_DIR=/tmp/tau-target-3623 cargo test -p tau-agent-core`
- `CARGO_TARGET_DIR=/tmp/tau-target-3623 cargo test -p tau-integration-tests --test agent_tool_memory_roundtrip`
- `CARGO_TARGET_DIR=/tmp/tau-target-3623 cargo test -p tau-tools mcp_server_runtime -- --test-threads=1`
- `CARGO_TARGET_DIR=/tmp/tau-target-3623 cargo test -p tau-ops mcp_sdk -- --test-threads=1`
- `CARGO_TARGET_DIR=/tmp/tau-target-3623 cargo test -p tau-skills install_skills -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/tau-target-3623 cargo test -p tau-skills skills_lockfile -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/tau-target-3623 cargo test -p tau-orchestrator plan_executor -- --test-threads=1`

Repo-level gate blockers observed during verification:
- `cargo fmt --check` fails on pre-existing workspace formatting drift outside this issue, including `crates/tau-ai`, `crates/tau-coding-agent`, `crates/tau-custom-command`, and `crates/tau-tui`.
- `CARGO_TARGET_DIR=/tmp/tau-target-3623 cargo clippy -p tau-agent-core -p tau-tools -p tau-skills -p tau-orchestrator -p tau-ops -- -D warnings` fails before reaching the touched crates cleanly because `tau-extensions` emits deprecated-item warnings under `-D warnings`.

## Risks / Mitigations
- Risk: action-history persistence gets duplicated if prompt-path finalization is
  added in the wrong layer.
  - Mitigation: extract one shared finalization helper rather than duplicating
    save logic in multiple callers.
- Risk: tool latency is measured inconsistently across serial/future-parallel
  paths.
  - Mitigation: record timing at the tool execution boundary and thread it into
    the history writer instead of recomputing downstream.
- Risk: MCP clients may already rely on facade success responses.
  - Mitigation: keep the response machine-readable and explicit; update docs in
    the same slice so the behavior change is intentional, not surprising.
- Risk: routing MCP skills through `tau-skills` surfaces latent trust/manifest
  failures.
  - Mitigation: add negative tests up front and treat those failures as desired
    policy enforcement rather than regressions.
- Risk: Slice 4 expands into a larger executor project.
  - Mitigation: require an explicit product decision before implementing new
    executor behavior; otherwise scope this issue to alignment/documentation.

## Interfaces / Contracts
- Action-history contract: completed `prompt*` and `continue_turn*` runs produce
  the same persistence side effect when action-history is enabled.
- Telemetry contract: persisted tool entries carry real `turn` and
  `latency_ms` values.
- MCP contract: stateful tools must be either side-effect-backed or explicitly
  not implemented.
- Skills contract: MCP callers see the same catalog/install/trust behavior as
  `tau-skills`.
- Orchestration contract: `plan_executor` naming/docs/public APIs cannot claim
  execution/scheduling behavior without tests proving it exists.

## ADR
No ADR is required if Slice 4 takes the documentation/API-alignment path and MCP
tool changes stay within the existing error-envelope conventions.

If implementation chooses a new public runtime contract for training/agent
lifecycle tooling or a real `plan_executor` execution API, add an ADR before
merging that slice because it crosses a public contract boundary.
