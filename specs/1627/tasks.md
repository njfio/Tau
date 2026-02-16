# Issue 1627 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): add failing test harness for rubric criteria presence, full
scoring-sheet coverage, deterministic output with fixed timestamp, and
fail-closed invalid fixture behavior.

T2: add rubric policy JSON and decision-matrix schema.

T3: add scaffold decision-matrix generator script producing JSON + Markdown
artifacts from deterministic default candidates.

T4: run generator to refresh checked-in artifacts and execute scoped
verification.

## Tier Mapping

- Functional: rubric criteria + full scoring-sheet coverage
- Conformance: deterministic JSON/Markdown outputs for fixed input
- Regression: invalid metadata/unresolved decision fail-closed behavior
