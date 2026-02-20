# Tasks: Issue #2717 - Implement Cortex runtime heartbeat loop, cross-session memory bulletin, and ArcSwap prompt injection

## Ordered Tasks
1. [x] T1 (RED): add failing conformance/regression tests for C-01..C-05 in `tau-agent-core` and `tau-gateway`.
2. [x] T2 (GREEN): implement `tau-agent-core` Cortex runtime struct with ArcSwap bulletin state and cross-session memory scan + summarize/fallback refresh logic.
3. [x] T3 (GREEN): integrate Cortex heartbeat loop lifecycle into gateway runtime startup/shutdown.
4. [x] T4 (GREEN): wire bulletin-injected system prompt composition into new-session initialization paths.
5. [x] T5 (REGRESSION): validate existing-session and auth behavior remains stable.
6. [x] T6 (VERIFY): run scoped fmt/clippy/targeted tests and capture C-06 evidence.
7. [x] T7 (DOC): update `tasks/spacebot-comparison.md` G3 checklist items completed by this slice.

## Tier Mapping
- Unit: Cortex prompt composition and fallback formatting helpers.
- Property: N/A (no new randomized invariant engine introduced).
- Contract/DbC: N/A (no contracts macro adoption in this slice).
- Snapshot: N/A (assert explicit fields/text behavior directly).
- Functional: C-02, C-04.
- Conformance: C-01..C-06.
- Integration: C-01, C-02, C-04.
- Fuzz: N/A (no new untrusted parser boundary introduced).
- Mutation: N/A (non-critical bounded feature slice; follow-up if designated critical path).
- Regression: C-03, C-05.
- Performance: N/A (no hotspot SLA/budget contract introduced in this slice).
