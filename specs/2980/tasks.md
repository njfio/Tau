# Tasks: Issue #2980 - OpenAI compatibility runtime extraction

1. [ ] T1 (RED): record baseline hotspot line-count and run failing/sanity scoped OpenAI compatibility test selection.
2. [ ] T2 (GREEN): extract OpenAI compatibility handlers (and required helper plumbing) into `openai_compat_runtime.rs`, then wire imports.
3. [ ] T3 (REGRESSION): run targeted OpenAI compatibility suites and nearby gateway regressions.
4. [ ] T4 (VERIFY): run fmt, clippy, and confirm hotspot reduction.
5. [ ] T5 (VALIDATE): run fast live validation process command for touched gateway contracts.

## Tier Mapping
- Unit: targeted gateway OpenAI compatibility tests.
- Property: N/A (no invariant algorithm changes).
- Contract/DbC: N/A (no contract macro additions).
- Snapshot: N/A (no snapshot outputs changed).
- Functional: OpenAI compatibility endpoint behavior checks.
- Conformance: C-01..C-04.
- Integration: route wiring + handler/runtime integration.
- Fuzz: N/A (no parser/fuzz target changes).
- Mutation: N/A (refactor-only move; no logic delta).
- Regression: scoped OpenAI compatibility and nearby gateway tests.
- Performance: N/A (no perf contract change).
