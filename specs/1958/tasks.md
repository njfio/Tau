# Issue 1958 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): add failing tests for retry metric presence/value and clean-run
absence, plus regression suite gate.

T2: add runner retry recovery context plumbing.

T3: emit deterministic retry metric rewards during recovered rollout processing.

T4: run scoped verification and map AC-1..AC-4 to C-01..C-04.

## Tier Mapping

- Unit: deterministic backoff value accounting
- Functional: clean-run no retry metrics
- Integration: transient failure recovery emits metrics
- Regression: existing runner/trainer suites remain green
- Conformance: C-01..C-04
