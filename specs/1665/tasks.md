# Issue 1665 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): add failing tests for span->trajectory adaptation, validation,
partial telemetry fallback, and empty-input deterministic error.

T2: implement trajectory adapter and extraction helpers.

T3: expose adapter from crate exports and update docs where needed.

T4: run scoped fmt/clippy/tests and verify AC evidence.

## Tier Mapping

- Unit: adapter helper extraction and ordering
- Functional: trajectory generation from representative span inputs
- Conformance: output validation against `EpisodeTrajectory::validate`
- Regression: deterministic empty-input/partial-telemetry behavior
