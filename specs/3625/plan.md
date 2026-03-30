# Plan: Issue #3625 - Make MCP lifecycle tools explicit about unsupported runtime operations

Status: Implemented
Milestone: M329
Parent: #3623

## Compatibility Strategy
```yaml
implementation_strategy:
  task: "3625"
  change_surface:
    - symbol: "tau.training_trigger MCP tool result contract"
      location: "crates/tau-tools/src/mcp_server_runtime.rs"
      change_type: "modification"
      current: "Returns isError=false with status=accepted despite no runtime action"
      proposed: "Returns isError=true with explicit not_implemented/runtime_unavailable contract"
      compatibility: "caution"
      reason: "Observable tool-call behavior changes for existing callers, but the prior contract was misleading."
    - symbol: "tau.agent_spawn/status/cancel MCP tool result contracts"
      location: "crates/tau-tools/src/mcp_server_runtime.rs"
      change_type: "modification"
      current: "Return placeholder accepted/unknown responses without runtime side effects"
      proposed: "Return isError=true with explicit not_implemented/runtime_unavailable contract"
      compatibility: "caution"
      reason: "Callers must stop treating these tools as successful in standalone MCP mode."
    - symbol: "Generated MCP SDK docs and built-in tool descriptions"
      location: "crates/tau-tools/src/mcp_server_runtime.rs, crates/tau-ops/src/mcp_sdk.rs"
      change_type: "modification"
      current: "Descriptions imply live trigger/spawn/cancel behavior"
      proposed: "Descriptions state runtime dependency / unsupported standalone behavior"
      compatibility: "safe"
      reason: "Documentation truth-in-advertising only."
  overall_compatibility: "caution"
  approach:
    strategy: "Direct implementation with compatibility tests"
    steps:
      - "Keep the tools visible, but return explicit error contracts when runtime wiring is absent."
      - "Use stable status/reason_code/message fields so callers can branch reliably."
      - "Update catalog/docs strings to stop implying guaranteed execution."
      - "Add regression coverage for the fake-success cases."
    migration_guide: |
      Callers should treat these MCP tools as unavailable in standalone MCP mode
      unless a future runtime-backed implementation changes the contract.
      Check `isError`, `status`, and `reason_code` instead of assuming
      `accepted` means work was scheduled.
    version_impact: "minor (pre-1.0) — behavioral correction to an inaccurate contract"
```

## Approach
1. Add RED coverage for the existing fake-success lifecycle responses and for
   tool descriptions that overclaim runtime behavior.
2. Introduce a shared helper for unsupported lifecycle operations so training
   and agent tools emit one stable contract.
3. Update built-in MCP tool descriptors and generated docs to match the runtime
   truth.
4. Verify with scoped `tau-tools` and `tau-ops` tests.

## Affected Modules
- `crates/tau-tools/src/mcp_server_runtime.rs`
- `crates/tau-ops/src/mcp_sdk.rs`

## Risks / Mitigations
- Risk: existing callers may rely on `"accepted"` or `"unknown"` strings.
  - Mitigation: keep the tools discoverable, use stable error fields, and note
    the compatibility change in the issue/process log.
- Risk: tests may still encode the old placeholder semantics in catalog-level
  assertions.
  - Mitigation: update those tests first so they fail before the runtime change.

## Verification Plan
- `cargo test -p tau-tools mcp_server_runtime -- --test-threads=1`
- `cargo test -p tau-ops mcp_sdk -- --test-threads=1`
- If scoped filtering is awkward, fall back to:
  - `cargo test -p tau-tools`
  - `cargo test -p tau-ops`

## Verification Result
- RED:
  - `CARGO_TARGET_DIR=/tmp/tau-target-3625 cargo test -p tau-tools regression_training_trigger_reports_runtime_unavailable_error -- --nocapture`
  - `CARGO_TARGET_DIR=/tmp/tau-target-3625 cargo test -p tau-ops regression_generate_mcp_tool_docs_marks_runtime_dependent_lifecycle_tools -- --nocapture`
- GREEN / VERIFY:
  - `rustfmt --check --edition 2021 crates/tau-tools/src/mcp_server_runtime.rs crates/tau-ops/src/mcp_sdk.rs`
  - `CARGO_TARGET_DIR=/tmp/tau-target-3625 cargo test -p tau-tools runtime_unavailable -- --nocapture`
  - `CARGO_TARGET_DIR=/tmp/tau-target-3625 cargo test -p tau-tools mcp_server_runtime -- --test-threads=1`
  - `CARGO_TARGET_DIR=/tmp/tau-target-3625 cargo test -p tau-ops mcp_sdk -- --test-threads=1`
- Note:
  - `cargo fmt -p tau-tools -p tau-ops --check` still reports unrelated
    pre-existing formatting drift elsewhere in `tau-ops`, so touched files were
    verified directly with `rustfmt --check`.

## ADR
No ADR required. This is a contract-honesty correction for existing tools, not
an architectural expansion.
