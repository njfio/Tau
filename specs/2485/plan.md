# Plan #2485

## Approach
1. Create M83 spec container and child issue artifacts before implementation.
2. Define bounded task behavior for heartbeat profile-policy watcher + ArcSwap config swaps.
3. Execute test-first implementation in #2487/#2488 with scoped verification and closure.

## Risks / Mitigations
- Risk: scope creep into full dynamic runtime config.
  Mitigation: keep updates limited to heartbeat scheduler policy fields.
- Risk: flaky watcher timing tests.
  Mitigation: assert deterministic state transitions with bounded polling helpers.

## Interfaces / Contracts
- Internal `tau-runtime` heartbeat hot-reload contract only.
- No public API/wire-format changes.
