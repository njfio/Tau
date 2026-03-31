# Plan: Issue #3657 - Add structured verifier bundles and first back-pressure adapters to gateway missions

## Approach
1. Introduce a gateway verifier runtime module that can build a structured
   verifier bundle from observed tool traces plus verifier requirements.
2. Extend mission supervisor persistence so each iteration stores the verifier
   bundle, while preserving a single top-level latest-verifier summary for quick
   status checks.
3. Replace the single tool-evidence stop/continue decision in the outer loop
   with a bundle-driven decision that can require:
   - tool evidence
   - mutating-work evidence
   - validation evidence
4. Use corrective retry prompts that enumerate unmet verifier requirements so
   the next iteration gets concrete back-pressure instead of a generic retry.

## Proposed Design
### Verifier bundle
- New `GatewayMissionVerifierBundle` stores:
  - `overall: GatewayMissionVerifierRecord`
  - `records: Vec<GatewayMissionVerifierRecord>`
- Overall verdict is derived from the record set:
  - any `failed` => overall failed
  - else any `continue` => overall continue
  - else passed

### First adapters
- `action_tool_evidence`: existing adapter retained as the base signal.
- `workspace_mutation_evidence`: active for prompts that request creating,
  editing, fixing, writing, generating, or otherwise changing workspace state.
- `validation_evidence`: active for prompts that explicitly request test,
  validate, verify, playable, or similar confirmation.

### Evidence sources
- Reuse the existing gateway tool-trace capture from the outer loop.
- Mutation evidence is satisfied by successful mutating tools such as
  `write`/`edit`, plus selected mutating bash commands.
- Validation evidence is satisfied by successful validation-oriented tools,
  initially focused on successful `bash` validation commands.
- Evaluation uses cumulative traces for the request so a later validation
  attempt can satisfy verification after earlier mutation work.

### Retry behavior
- Continue verdicts append a verifier-driven corrective prompt.
- Failed verdicts block the mission with the verifier bundle's overall reason.
- Passed verdicts allow the current response to complete normally.

## Compatibility Assessment
```yaml
implementation_strategy:
  task: "3657"
  change_surface:
    - symbol: "Gateway mission iteration persistence"
      location: "crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs"
      change_type: "modification"
      current: "each iteration stores a single verifier record"
      proposed: "each iteration stores a verifier bundle plus an overall summary"
      compatibility: "caution"
      reason: "changes persisted mission JSON shape but only for gateway-local internal state"
    - symbol: "Gateway outer-loop completion policy"
      location: "crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs"
      change_type: "modification"
      current: "tool execution alone can satisfy action-oriented completion"
      proposed: "completion requires the active verifier bundle to pass"
      compatibility: "caution"
      reason: "changes runtime behavior for action-oriented prompts without changing HTTP schema"
  overall_compatibility: "caution"
  approach:
    strategy: "Additive verifier-bundle rollout inside the gateway executor"
    steps:
      - "Introduce verifier bundle/runtime helpers"
      - "Persist bundle state per mission iteration"
      - "Drive retries from verifier bundle outcomes"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: verifier heuristics over-constrain prompts and create retry churn.
  Mitigation: scope mutation/validation adapters to explicit prompt intents and
  use bounded retries with specific corrective feedback.
- Risk: bundle persistence complicates existing mission-state assertions.
  Mitigation: preserve a top-level overall verifier summary alongside the full
  bundle.
- Risk: validation detection becomes too tied to one tool.
  Mitigation: keep the first slice explicit and small, then extend adapters in
  later stories once the contract is proven.

## Verification
- Unit tests for verifier bundle aggregation
- Regression coverage for mutation-driven retries
- Regression coverage for validation-driven retries
- Existing mission/session/learning gateway tests remain green
