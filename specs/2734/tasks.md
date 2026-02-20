# Tasks: Issue #2734 - G18 stretch routines cron management webchat panel

## Ordered Tasks
1. [x] T1 (RED): add failing webchat routines conformance tests for tab/markers/request handlers.
2. [x] T2 (GREEN): add routines tab/view markup and controls.
3. [x] T3 (GREEN): implement routines status renderer and jobs list/cancel handlers.
4. [x] T4 (REGRESSION): verify existing webchat and jobs/status endpoint tests remain green.
5. [x] T5 (VERIFY): run scoped fmt/clippy/targeted gateway tests.
6. [x] T6 (DOC): update G18 cron-management checklist evidence in `tasks/spacebot-comparison.md`.

## Tier Mapping
- Unit: C-01, C-02
- Property: N/A (no randomized invariant harness introduced)
- Contract/DbC: N/A (no contracts macro changes)
- Snapshot: N/A (explicit marker assertions)
- Functional: C-03
- Conformance: C-01..C-05
- Integration: C-03, C-04
- Fuzz: N/A (no new untrusted parser boundary)
- Mutation: N/A (UI parity slice, non-critical mutation lane)
- Regression: C-04
- Performance: N/A (no benchmark SLA introduced)
