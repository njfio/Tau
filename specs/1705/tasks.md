# Issue 1705 Tasks

Status: Implementing

## Ordered Tasks

T1 (tests-first): run scanner and capture baseline stale findings as RED evidence.

T2: remediate stale findings through policy/docs updates with constrained scope.

T3: regenerate scan artifacts and verify `stale_count == 0`.

T4: run scanner + allowlist contract tests for regression safety.

## Tier Mapping

- Functional: scanner execution and artifact generation
- Conformance: stale count reaches zero
- Integration: policy/guide contract alignment checks
- Regression: scanner fixture/error-path tests stay green
