# Issue 1651 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): add script contract test fixture and RED checks for expected
counts, JSON fields, and fail-closed error behavior.

T2: implement `rustdoc-marker-count.sh` with stdout summary and JSON/Markdown
artifact output.

T3: update doc-density scorecard guide with command usage and purpose.

T4: run script tests and generate baseline `tasks/reports` artifacts.

## Tier Mapping

- Functional: fixture-based total/per-crate count checks
- Conformance: JSON output schema and stable sorted crate report
- Integration: scorecard documentation references new command
- Regression: unknown flag and missing scan root fail non-zero
