# Issue 1634 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): add `scripts/dev/test-m21-tool-split-validation.sh` and capture RED before combined runner exists.

T2: implement `scripts/dev/m21-tool-split-validation.sh` to run parity + perf scripts and produce combined JSON/markdown artifacts.

T3: run GREEN harness and regenerate canonical report artifacts.

T4: run scoped roadmap/fmt/clippy checks and prepare PR evidence.

## Tier Mapping

- Functional: combined runner pass path and artifact output
- Regression: deterministic warn/fail semantics under fixture drift/failure
- Integration: combined harness + scoped quality checks
