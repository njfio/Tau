# Plan: Issue #3655 - Implement the first Tau Ralph supervisor loop slice for gateway missions

## Approach
1. Extend request translation with an additive `mission_id` concept:
   - explicit from `metadata.mission_id` when present
   - implicit from the resolved session key otherwise
2. Add a gateway-local mission-supervisor persistence module under
   `state_dir/openresponses/missions/` with:
   - one JSON file per mission
   - schema version
   - mission/session linkage
   - outer-loop iteration history
   - terminal status and latest verifier outcome
3. Wrap the existing gateway execution path so the current outer retry loop
   becomes the first durable supervisor loop:
   - create/load mission state before the first attempt
   - persist an iteration record after each attempt
   - mark the mission `completed` or `blocked` at terminal exit
4. Keep the compatibility boundary additive:
   - no new required request fields
   - no response schema break
   - existing session persistence remains the source of transcript lineage

## Proposed Data Model
- `GatewayMissionState`
  - `schema_version`
  - `mission_id`
  - `session_key`
  - `response_id`
  - `goal_summary`
  - `status`
  - `created_unix_ms`
  - `updated_unix_ms`
  - `iteration_count`
  - `latest_verifier`
  - `iterations[]`
- `GatewayMissionIterationRecord`
  - `attempt`
  - `prompt_summary`
  - `tool_execution_count`
  - `assistant_summary`
  - `verifier`
  - `started_unix_ms`
  - `finished_unix_ms`
- `GatewayMissionVerifierRecord`
  - `kind`
  - `status`
  - `reason_code`
  - `message`

## Compatibility Assessment
```yaml
implementation_strategy:
  task: "3655"
  change_surface:
    - symbol: "OpenResponses request metadata"
      location: "crates/tau-gateway/src/gateway_openresponses/request_translation.rs"
      change_type: "additive"
      current: "metadata supports session_id and unrelated passthrough fields"
      proposed: "metadata may also include optional mission_id"
      compatibility: "safe"
      reason: "mission_id is optional and falls back to the current session-derived behavior"
    - symbol: "Gateway request execution persistence"
      location: "crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs"
      change_type: "modification"
      current: "request-local retry loop with no mission persistence"
      proposed: "persist mission supervisor state around the existing retry loop"
      compatibility: "caution"
      reason: "observable side effect adds durable mission files but does not change request/response schema"
  overall_compatibility: "caution"
  approach:
    strategy: "Direct implementation with additive persistence"
    steps:
      - "Resolve mission_id during request translation"
      - "Persist mission state in a new gateway-local mission store"
      - "Record verifier-backed iteration outcomes around the existing retry loop"
      - "Keep all existing response semantics intact"
    version_impact: "none"
```

## Affected Areas
- `crates/tau-gateway/src/gateway_openresponses/request_translation.rs`
- `crates/tau-gateway/src/gateway_openresponses/types.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks / Mitigations
- Risk: mission files grow without bound from raw prompt/output storage.
  Mitigation: persist concise summaries rather than full transcripts.
- Risk: mission state and session state diverge.
  Mitigation: store explicit `session_key` linkage and keep transcripts in the
  existing session store only.
- Risk: the first slice looks like a second retry system rather than the start
  of the canonical outer loop.
  Mitigation: make the persisted iteration model describe verifier outcomes and
  terminal mission status, not just retry counters.

## Verification
- Unit coverage for mission-id translation
- Regression coverage for successful retry persistence and exhausted retry
  persistence
- Existing gateway retry/tool-pipeline tests stay green
