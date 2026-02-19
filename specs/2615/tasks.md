# Tasks: Issue #2615 - Integrate RL loop with live agent decision-making path

## Ordered Tasks
1. T1 (RED): add failing tests for live event collection, optimizer scheduling, and failure-gate hold transitions.
2. T2 (GREEN): implement `live_rl_runtime` bridge and wire startup registration.
3. T3 (VERIFY): run scoped fmt/clippy/tests and validate AC/C mappings.
4. T4 (CLOSE): update issue process log and open PR with tier matrix + TDD evidence.

## Tier Mapping
- Unit: C-04
- Property: N/A (deterministic event/state transitions)
- Contract/DbC: N/A (no contracts macro usage)
- Snapshot: N/A (explicit structured assertions)
- Functional: C-01, C-02
- Conformance: C-01..C-05
- Integration: C-01 (agent event subscription -> store persistence path)
- Fuzz: N/A (no new parser/untrusted input surface)
- Mutation: N/A (staged runtime integration slice)
- Regression: C-03
- Performance: N/A (opt-in disabled-by-default path)
