# Plan: Issue #3658 - Add explicit mission completion and checkpoint semantics to the gateway Ralph loop

## Approach
1. Introduce a gateway-internal `complete_task` tool that the agent can call
   with `success`, `partial`, or `blocked` plus a summary.
2. Parse explicit completion signals from observed tool traces in the outer
   gateway loop without changing the public HTTP contract.
3. Extend mission supervisor persistence so each iteration can store the latest
   explicit completion signal and mission state can include `checkpointed`.
4. Keep the rollout compatibility-safe:
   - verifier pass without `complete_task` still completes as before
   - explicit `partial` or `blocked` signals override inferred completion

## Proposed Design
### Internal completion tool
- Register `complete_task` on every gateway mission agent in addition to the
  existing external tool registrar.
- Tool arguments:
  - `summary: string`
  - `status: "success" | "partial" | "blocked"` default `success`
  - optional `next_step: string`
- Tool result is successful structured JSON so the trace can be parsed later.

### Mission state
- Add `GatewayMissionStatus::Checkpointed`
- Add `GatewayMissionCompletionStatus` and
  `GatewayMissionCompletionSignalRecord`
- Persist the latest completion signal on the mission and per iteration

### Outer-loop behavior
- If an explicit completion signal with `partial` is observed, stop the outer
  loop and mark the mission `checkpointed`
- If an explicit completion signal with `blocked` is observed, stop the outer
  loop and mark the mission `blocked`, but return a normal response containing
  the completion summary
- If an explicit completion signal with `success` is observed and verifier
  requirements pass, mark the mission `completed`
- If no explicit completion signal is observed, retain the current verifier-only
  compatibility behavior

## Compatibility Assessment
```yaml
implementation_strategy:
  task: "3658"
  change_surface:
    - symbol: "Gateway mission tool inventory"
      location: "crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs"
      change_type: "modification"
      current: "gateway missions expose only the configured gateway tool registrar tools"
      proposed: "gateway missions also expose an internal complete_task tool"
      compatibility: "caution"
      reason: "changes model-visible tool inventory but not the external HTTP schema"
    - symbol: "Gateway mission persistence"
      location: "crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs"
      change_type: "modification"
      current: "mission state distinguishes running/completed/blocked only"
      proposed: "mission state also captures checkpointed and explicit completion signals"
      compatibility: "caution"
      reason: "changes internal persisted mission JSON shape"
  overall_compatibility: "caution"
  approach:
    strategy: "Additive internal completion contract with compatibility fallback"
    steps:
      - "Register complete_task as an internal gateway tool"
      - "Parse completion signals from tool traces"
      - "Persist completion signals and checkpointed state"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: the model ignores the new tool initially.
  Mitigation: preserve verifier-only compatibility while the tool is introduced.
- Risk: explicit blocked/partial outcomes get confused with runtime failures.
  Mitigation: treat them as intentional mission outcomes with success-shaped HTTP
  responses and explicit mission-state markers.
- Risk: mission persistence becomes harder to inspect.
  Mitigation: keep status, latest verifier, and latest completion signal as
  separate top-level fields.

## Verification
- Unit coverage for completion-signal extraction
- Regression coverage for partial checkpoint, explicit blocked, and explicit
  success completion
- Existing verifier/learning/session gateway tests remain green
