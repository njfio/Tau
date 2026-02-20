# Tasks: Issue #2750 - Discord guild allowlist filtering for live connectors (G10)

## Ordered Tasks
1. [x] T1 (RED): add failing tests for CLI defaults/overrides, validation, startup mapping, and Discord polling guild filtering.
2. [x] T2 (GREEN): add CLI arg/env + validation for Discord guild allowlist IDs.
3. [x] T3 (GREEN): propagate guild allowlist into live connector config and enforce filtering in polling ingestion.
4. [x] T4 (REGRESSION): confirm behavior unchanged when guild allowlist is unset.
5. [x] T5 (VERIFY): run fmt, clippy, and targeted test suites plus live localhost validation.
6. [x] T6 (DOC): update `tasks/spacebot-comparison.md` G10 checklist evidence.

## Tier Mapping
- Unit: C-01, C-02
- Property: N/A (no randomized invariant surface)
- Contract/DbC: N/A (no contract-macro changes)
- Snapshot: N/A (assertive behavior tests only)
- Functional: C-02, C-04
- Conformance: C-01..C-06
- Integration: C-03, C-05
- Fuzz: N/A (no new untrusted parser boundary)
- Mutation: N/A (scoped connector filtering change)
- Regression: C-05
- Performance: N/A (no benchmarked hotspot changes)
