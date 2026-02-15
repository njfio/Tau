# Issue 1725 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): add failing concurrent collector load-harness regression test
that enqueues a burst and expects no-drop terminal completion.

T2: implement harness metrics emission (elapsed + throughput) and deterministic
assertions.

T3: add developer script entrypoint to run harness and surface metrics output.

T4: run fmt/clippy/tests for touched crates and verify AC mapping.

## Tier Mapping

- Functional: metrics emitted from harness
- Integration: concurrent worker burst run
- Regression: no silent drop invariant
