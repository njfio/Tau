# Issue 1964 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): add failing tests for clean batch, multi-attempt batch,
windowed batch, and unknown rollout error.

T2: add collection report types and helper implementation.

T3: wire optional window policy and deterministic skip reasoning.

T4: run scoped verification and map AC-1..AC-4 to C-01..C-04.

## Tier Mapping

- Unit: unknown rollout deterministic error
- Functional: single-rollout deterministic batch
- Integration: multi-attempt retry/requeue batch
- Conformance: window policy behavior through collection helper
- Regression: tau-algorithm suite remains green
