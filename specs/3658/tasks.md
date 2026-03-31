# Tasks: Issue #3658 - Add explicit mission completion and checkpoint semantics to the gateway Ralph loop

- [x] T1 (RED): add unit coverage for extracting explicit completion signals
      from `complete_task` tool traces.
- [x] T2 (RED): add regression coverage proving `partial` completion persists a
      checkpointed mission state.
- [x] T3 (RED): add regression coverage proving `blocked` completion persists a
      blocked mission state and returns a success-shaped response.
- [x] T4 (RED): add regression coverage proving `success` completion is
      persisted when verifier requirements already pass.
- [x] T5 (GREEN): add the gateway `complete_task` tool and completion-signal
      parsing helpers.
- [x] T6 (GREEN): persist explicit completion signals and checkpointed mission
      state in mission supervisor records.
- [x] T7 (GREEN): integrate completion signals into the gateway outer-loop stop
      logic with compatibility fallback.
- [x] T8 (VERIFY): run targeted `tau-gateway` completion, verifier, learning,
      and mission persistence verification.

## Tier Mapping
- Unit: completion-signal extraction and parsing
- Regression: partial checkpoint, explicit blocked, and explicit success
  completion flows
- Functional: gateway responses remain usable while mission state records the
  explicit outcome
- Integration: prior verifier, learning, and session-roundtrip flows stay green
