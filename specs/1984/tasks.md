# Issue 1984 Tasks

Status: Implementing

## Ordered Tasks

T1 (tests-first): add failing tests for combined summary report pass path,
reason-code propagation, report JSON shape, and invalid-policy failure.

T2: add summary gate report model and builder helper.

T3: wire report serialization with nested `summary` + `quality` sections.

T4: run scoped verification and map AC-1..AC-4 to C-01..C-04.

## Tier Mapping

- Unit: invalid policy fail-closed behavior
- Functional: deterministic pass-path summary report
- Integration: failing reason-code propagation
- Conformance: machine-readable summary report JSON shape
- Regression: zero-summary report stability
