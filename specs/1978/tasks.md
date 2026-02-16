# Issue 1978 Tasks

Status: Implementing

## Ordered Tasks

T1 (tests-first): add failing tests for deterministic gate report export,
validator pass path, validator malformed-json rejection, and file-destination
export failure.

T2: add gate report export helper with deterministic filename + summary output.

T3: add replay validator helper for exported gate report payloads.

T4: run scoped verification and map AC-1..AC-4 to C-01..C-04.

## Tier Mapping

- Unit: malformed/non-object validator rejection behavior
- Functional: deterministic gate report export path/summary
- Conformance: validator accepts exported payload with required sections
- Regression: export rejects file destination path
