# Plan #2488

## Approach
1. Add C-01..C-04 tests with `spec_2487` naming in heartbeat runtime module.
2. Run scoped tests before implementation and capture failing output (RED).
3. Implement #2487 behavior.
4. Re-run same scoped tests and capture passing output (GREEN).

## Risks / Mitigations
- Risk: RED does not fail due to incomplete assertions.
  Mitigation: assert watcher-triggered behavior not present in current implementation.

## Interfaces / Contracts
- No new interfaces; evidence-only subtask bound to #2487.
