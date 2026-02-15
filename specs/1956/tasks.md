# Issue 1956 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): add failing tests for bounded backoff math, config validation,
fail->success->fail reset semantics, and transient recovery integration.

T2: add retry config fields and validation helpers.

T3: wire poll-path retry/backoff in runner loop.

T4: run scoped verification and map AC-1..AC-4 to C-01..C-04.

## Tier Mapping

- Unit: backoff math + config validation
- Integration: transient failure then success rollout completion
- Regression: backoff reset after successful poll
- Conformance: C-01..C-04
