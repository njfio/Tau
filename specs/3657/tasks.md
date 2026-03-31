# Tasks: Issue #3657 - Add structured verifier bundles and first back-pressure adapters to gateway missions

- [x] T1 (RED): add unit coverage for verifier bundle aggregation across tool,
      mutation, and validation requirements.
- [x] T2 (RED): add regression coverage proving workspace-changing prompts retry
      until successful mutating evidence is observed.
- [x] T3 (RED): add regression coverage proving validation-requesting prompts
      retry until successful validation evidence is observed.
- [x] T4 (GREEN): add the gateway verifier runtime and bundle model.
- [x] T5 (GREEN): persist verifier bundles in mission supervisor state.
- [x] T6 (GREEN): drive gateway outer-loop retries from verifier bundle
      outcomes and verifier-specific corrective prompts.
- [x] T7 (VERIFY): run targeted `tau-gateway` verifier, mission, retry,
      learning, and session-roundtrip verification.

## Tier Mapping
- Unit: verifier bundle aggregation and classifier behavior
- Regression: mutation-evidence and validation-evidence retry flows
- Functional: action-oriented gateway prompt completes only after active
  verifier requirements pass
- Integration: prior mission persistence, learning injection, and
  session-roundtrip flows remain green
