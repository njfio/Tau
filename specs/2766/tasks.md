# Tasks: Issue #2766 - Discord thread creation command and provider typing dispatch (G10)

## Ordered Tasks
1. [x] T1 (RED): add failing tests for outbound thread creation + runtime `/tau thread` + typing dispatch integration.
2. [x] T2 (GREEN): implement outbound Discord thread and typing dispatch methods.
3. [x] T3 (GREEN): implement runtime command parser/execution for `/tau thread` and logging payloads.
4. [x] T4 (REGRESSION): confirm existing non-Discord command and typing lifecycle behavior remains stable.
5. [x] T5 (VERIFY): run fmt, clippy, targeted tests, and local live validation.
6. [x] T6 (DOC): update G10 implementation checklist row with issue evidence.

## Tier Mapping
- Unit: C-02 parser/render coverage
- Property: N/A (no randomized invariant surface)
- Contract/DbC: N/A (no contract macro changes)
- Snapshot: N/A (assertive behavior tests)
- Functional: C-02
- Conformance: C-01..C-06
- Integration: C-01, C-03
- Fuzz: N/A (no new parser boundary)
- Mutation: N/A (scoped adapter/runtime flow with explicit conformance coverage)
- Regression: C-04
- Performance: N/A (no benchmarked hotspot changes)
