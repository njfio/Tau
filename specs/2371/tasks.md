# Issue 2371 Tasks — Typed Memory + Importance Foundation

## Ordered Tasks

1. T1 (Conformance tests first)
   - Add failing tests for C-01..C-05 in `tau-tools` memory tool tests.
   - Tiers: Conformance, Unit, Functional, Integration, Regression.
   - Dependency: none.

2. T2 (Runtime type + persistence changes)
   - Add `MemoryType` and default importance mappings in `tau-memory`.
   - Add persisted record fields with serde defaults.
   - Add write API accepting optional type/importance with defaults.
   - Dependency: T1.

3. T3 (Tool argument + output wiring)
   - Add `memory_type`/`importance` schema and parser validation.
   - Propagate metadata through write/read/search tool outputs.
   - Dependency: T2.

4. T4 (Ranking integration)
   - Apply importance boost in search ranking score computation.
   - Ensure deterministic ordering behavior for conformance case C-04.
   - Dependency: T3.

5. T5 (Verify and capture evidence)
   - Run scoped fmt/clippy/tests.
   - Capture RED/GREEN command excerpts and tier matrix for PR.
   - Dependency: T4.

## Tier Coverage Plan

- Unit: argument validation + default mapping checks.
- Functional: write/read round-trip metadata behavior.
- Conformance: C-01..C-05 tests.
- Integration: end-to-end memory tool search ordering behavior.
- Regression: default/legacy behavior remains valid.
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for this scoped
  slice unless touched logic reveals a justified need during implementation.
