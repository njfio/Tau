# Tasks: Issue #3655 - Implement the first Tau Ralph supervisor loop slice for gateway missions

- [ ] T1 (RED): add translation tests for explicit and implicit `mission_id`
      resolution.
- [ ] T2 (RED): add gateway regression tests proving successful retry missions
      persist ordered iteration history and terminal `completed` status.
- [ ] T3 (RED): add gateway regression tests proving exhausted retry missions
      persist terminal `blocked` status with the latest verifier reason.
- [ ] T4 (GREEN): add mission-supervisor persistence types/helpers and wire
      request translation to resolve `mission_id`.
- [ ] T5 (GREEN): integrate mission-supervisor state updates into the gateway
      outer loop without breaking session persistence or response semantics.
- [ ] T6 (VERIFY): run targeted `tau-gateway` tests covering translation,
      mission persistence, zero-tool retry recovery, and regression safety.

## Tier Mapping
- Unit: mission-id translation resolution
- Functional: implicit single-mission compatibility path
- Regression: retry-success and retry-exhaustion mission persistence
- Integration: existing gateway request/session roundtrip remains green
