# Issue 1990 Tasks

Status: Implementing

## Ordered Tasks

T1 (tests-first): add failing tests for pass path, fail reason propagation,
machine-readable decision JSON, and invalid-policy rejection.

T2: add manifest quality policy/decision models and evaluator helper.

T3: add quality decision JSON projection.

T4: run scoped verification and map AC-1..AC-4 to C-01..C-04.

## Tier Mapping

- Unit: invalid policy fail-closed behavior
- Functional: passing manifest decision path
- Integration: failing manifest reason-code propagation
- Conformance: machine-readable decision JSON payload
- Regression: zero-entry manifest ratio handling
