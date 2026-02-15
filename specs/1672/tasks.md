# Issue 1672 Tasks

Status: Reviewed

## Ordered Tasks

T1 (tests-first): add failing safety regression benchmark script tests for
pass/fail threshold paths.

T2: implement safety regression benchmark generator with deterministic reason
codes and delta reporting.

T3: integrate safety benchmark output into live benchmark proof generation.

T4: update docs with safety benchmark and integration usage.

T5: run scoped verification and map AC-1..AC-4 to C-01..C-04.

## Tier Mapping

- Unit: input parsing and threshold decision assertions
- Functional: pass and blocked benchmark report generation
- Integration: live proof flow consumes safety benchmark report
- Regression: threshold-breach path exits non-zero with deterministic reason code
- Conformance: C-01..C-04
